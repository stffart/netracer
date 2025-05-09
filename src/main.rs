use clap::Parser;
use ctrlc;
extern crate pcap;
extern crate pnet;
extern crate chrono;

use chrono::prelude::DateTime;
use chrono::Utc;

use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::Packet;


use native_db::{Models, Builder, Database};
use native_db::transaction::query::PrimaryScanIterator;

use std::time::{SystemTime};
use include_dir::{include_dir, Dir};

use rust_xlsxwriter::*;

use actix_cors::Cors;
use actix_web::{rt, get, App, HttpRequest, HttpResponse, HttpServer, Responder, 
                dev::ServiceRequest, error::ErrorUnauthorized, Error as ActixError, middleware::Condition};
use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web_httpauth::{extractors::basic::BasicAuth, middleware::HttpAuthentication};

use std::process;
use openssl::ssl::{SslAcceptor, SslMethod, SslFiletype};

use rcgen::{generate_simple_self_signed, CertifiedKey};

static UI_DIR: Dir<'_> = include_dir!("ntfront");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Network interface to listen on (e.g. eth0)
    #[arg(short, long)]
    interface: String,
    /// Listens to HTTPS on port tcp/3095. If not specified, then HTTP is listened to on the same port.)
    #[arg(short, long, default_value_t = false)]
    tls: bool,
    /// For TLS - certificate file name
    #[arg(short, long)]
    cert: Option<String>,
    /// For TLS - key file name
    #[arg(short, long)]
    key: Option<String>,
    /// Enables basic authentication by name and password. Specify the path to the file created using htpasswd.
    #[arg(short, long)]
    authfile: Option<String>,
    /// UDP Connections to destination ports above this will not be registered (for example filter out IANA private ports -d 49152)
    #[arg(short='d', long, default_value_t = 65535)]
    max_dst_udp_port: u16,
    /// UDP Connections from source ports below this will not be registered (default 1-2048 usually server answers) 
    #[arg(short='s', long, default_value_t = 2048)]
    min_src_udp_port: u16
}


pub mod data {
    use native_db::{native_db, ToKey, Key};
    use native_model::{native_model, Model};
    use serde::{Deserialize, Serialize};

    pub type Connection = v1::Connection;
    pub type Address = v1::Address;

    pub mod v1 {
        use super::*;         
        #[derive(Debug, Deserialize, Serialize, Clone)]
        pub struct Address {
          pub src: String,
          pub dst: String,      
          pub protocol: String,    
          pub port: String
        }

        impl ToKey for Address {
           fn to_key(&self) -> Key {
             let keystr = format!("{}_{}_{}_{}",self.src, self.dst, self.protocol, self.port);
             Key::new(keystr.as_bytes().to_vec())
           }

           fn key_names() -> Vec<String> {
             vec!["Address".to_string()]
           }
        }


        #[derive(Serialize, Deserialize, Debug, Clone)]
        #[native_model(id = 1, version = 1)]
        #[native_db]
        pub struct Connection {
          #[primary_key]
          pub addr: Address,
          pub time: u64,
          pub max_speed: u32,
          pub avg_speed: u32
        }
    }
}

use once_cell::sync::Lazy;

static MODELS: Lazy<Models> = Lazy::new(|| {
   let mut models = Models::new();
   // It's a good practice to define the models by specifying the version
   models.define::<data::v1::Connection>().unwrap();
   models
});

static DB: Lazy<Database> = Lazy::new(|| {
  let db =  Builder::new().create(&MODELS,"/var/netracer.ndb").unwrap();
  db
});
        

#[get("/{filename:.*}")]
async fn mainpage(req: HttpRequest) -> impl Responder {
    let mut path = req.match_info().query("filename");
    println!("{}",path);
    if path == "" {
       path = "index.html";
    }    
    let file = UI_DIR.get_file(path).unwrap();
//    let body = file.contents_utf8().unwrap();
    let body = file.contents();
    HttpResponse::Ok().body(body)
}

fn get_connections() -> Vec<data::Connection> {
    let r = DB.r_transaction().unwrap();
    let binding = r.scan().primary().unwrap();
    let mut values: PrimaryScanIterator<data::Connection> = binding.all().unwrap();
    let mut cons: Vec<data::Connection> = Vec::new();
    while let Some(con) = values.next() {
      match con {
        Ok(c) => cons.push(c),
        Err(_e) => {}
      }
    }
    cons
}

#[get("/con")]
async fn connections() -> impl Responder {
    let cons = get_connections();
    HttpResponse::Ok().json(cons)
}

fn get_connections_agg() -> Vec<data::Connection> {
    let r = DB.r_transaction().unwrap();
    let binding = r.scan().primary().unwrap();
    let mut values: PrimaryScanIterator<data::Connection> = binding.all().unwrap();
    // aggregate by source and port
    let mut cons: Vec<data::Connection> = Vec::new();
    while let Some(con) = values.next() {
      match con {
        Ok(c) => { 
           let c1 = c.clone();
           let mut found = false;
           for con0 in &mut cons {
             if con0.addr.port == c.addr.port && 
                con0.addr.protocol == c.addr.protocol &&
                con0.addr.src == c.addr.src 
                {
                   con0.addr.dst = format!("{} {}",con0.addr.dst, c.addr.dst);
                   con0.time = std::cmp::max(con0.time, c.time);
                   found = true;
                   break;
                }
           }
           if !found {
             cons.push(c1);
           }
        },
        Err(_e) => {}
      }
    }

    // aggregate by source and destination
    let mut cons2: Vec<data::Connection> = Vec::new();
    for c in cons {
       let mut found = false;
       for con0 in &mut cons2 {
         if con0.addr.src == c.addr.src &&
            con0.addr.dst == c.addr.dst &&
            con0.addr.protocol == c.addr.protocol
            {
              con0.addr.port = format!("{}, {}", con0.addr.port, c.addr.port);
              con0.time = std::cmp::max(con0.time, c.time);
              found = true;
              break;
            }
       }
       if !found {
         cons2.push(c);
       }
    }

    // aggregate by destination and port
    let mut cons3: Vec<data::Connection> = Vec::new();
    for c in cons2 {
       let mut found = false;
       for con0 in &mut cons3 {
         if con0.addr.port == c.addr.port && 
            con0.addr.protocol == c.addr.protocol &&
            con0.addr.dst == c.addr.dst 
            {
              con0.addr.src = format!("{} {}", con0.addr.src, c.addr.src);
              con0.time = std::cmp::max(con0.time, c.time);
              found = true;
              break;
            }
       }
       if !found {
         cons3.push(c);
       }
    }
    cons3
}

#[get("/conagg")]
async fn connections_agg() -> impl Responder {
    let cons = get_connections_agg();
    HttpResponse::Ok().json(cons)
}

fn export_xls(cons: Vec<data::Connection>, filename: &str) {
    let mut workbook = Workbook::new();
    let sheet1 = workbook.add_worksheet();    
    let mut n: u32 = 1;
    let cell_format: &Format = &Format::new().set_text_wrap();
    let header_format: &Format = &Format::new().set_text_wrap().set_background_color(Color::Gray);
    sheet1.set_column_width(0, 50 as f64).unwrap();
    sheet1.set_column_width(1, 50 as f64).unwrap();
    sheet1.set_column_width(3, 50 as f64).unwrap();
    sheet1.set_column_width(4, 50 as f64).unwrap();
    sheet1.write_string_with_format(0, 0, "Source", header_format).unwrap();
    sheet1.write_string_with_format(0, 1, "Destination", header_format).unwrap();
    sheet1.write_string_with_format(0, 2, "Protocol", header_format).unwrap();
    sheet1.write_string_with_format(0, 3, "Ports", header_format).unwrap();
    sheet1.write_string_with_format(0, 4, "LastDate", header_format).unwrap();
    for con in cons {
      sheet1.write_string_with_format(n, 0, &*str::replace(&*con.addr.src," ","\n"), cell_format).unwrap();
      sheet1.write_string_with_format(n, 1, &*str::replace(&*con.addr.dst," ","\n"), cell_format).unwrap();
      sheet1.write_string(n, 2, &*con.addr.protocol).unwrap();
      sheet1.write_string(n, 3, &*con.addr.port).unwrap();
      let datetime: DateTime<Utc> = DateTime::from_timestamp(con.time as i64, 0).unwrap();
      let newdate = datetime.format("%d.%m.%Y %H:%M");
      let newdate_str = format!("{}",newdate);
      sheet1.write_string_with_format(n, 4, &*newdate_str, cell_format).unwrap();
      n = n + 1;
    }
    workbook.save(filename).unwrap();    
}

#[get("/conaggxls")]
async fn connections_agg_xls() -> Result<NamedFile, actix_web::Error> {
    const FILENAME: &str = "/var/netracer_agg.xlsx";
    let cons = get_connections_agg();
    export_xls(cons, FILENAME);
    let file = NamedFile::open(FILENAME).unwrap();
    Ok(file.use_last_modified(true).set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

#[get("/conxls")]
async fn connections_xls() -> Result<NamedFile, actix_web::Error> {
    const FILENAME: &str = "/var/netracer.xlsx";
    let cons = get_connections();
    export_xls(cons, FILENAME);
    let file = NamedFile::open(FILENAME).unwrap();
    Ok(file.use_last_modified(true).set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

async fn do_auth(
      req: ServiceRequest,
      creds: BasicAuth,
      ) -> Result<ServiceRequest, (ActixError, ServiceRequest)> {
     let args: Args = Args::parse();
     let authfile: Option<String> = args.authfile;
     if authfile != None {
         let htpasswd_data: String = std::fs::read_to_string(authfile.unwrap()).unwrap();
         let htpasswd: htpasswd_verify::Htpasswd<'_> = htpasswd_verify::Htpasswd::from(&*htpasswd_data);
         let pass = creds.password();
         if pass != None {
           if htpasswd.check(creds.user_id(), creds.password().unwrap()) { 
             Ok(req)
           } else {
              Err((ErrorUnauthorized("Unauthorized"), req))
           }
         } else
         {
              Err((ErrorUnauthorized("Unauthorized"), req))
         }
     } else {
         Ok(req)
     }
}


#[actix_web::main]
async fn main() -> Result<(), native_db::db_type::Error> {
    let args: Args = Args::parse();
    println!("{:?}", args);
    let tls = args.tls;
    let cert_file = args.cert;
    let key_file = args.key;


    let authfile: Option<String> = args.authfile;

    let new_srv = HttpServer::new(move || {App::new().wrap(Cors::permissive()).
                                     wrap(Condition::new(authfile != None, HttpAuthentication::basic(do_auth))).
                                  service(connections).service(connections_agg).
                                  service(connections_xls).service(connections_agg_xls).service(mainpage)
                                 });
    let srv: actix_web::dev::Server;
    if tls {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        if cert_file == None && key_file == None {
          println!("Certificate (--cert) and key (--key) not specified. Generating self-signed...");
          let subject_alt_names = vec!["netracer.local".to_string(),
                                       "localhost".to_string()];
          let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names).unwrap();

          let pkey = openssl::pkey::PKey::private_key_from_pem(key_pair.serialize_pem().as_bytes()).unwrap();
          builder.set_private_key(pkey.as_ref()).unwrap();
          let x509 = openssl::x509::X509::from_pem(cert.pem().as_bytes()).unwrap();
          builder.set_certificate(x509.as_ref()).unwrap();
        } else {
          if cert_file == None {
            println!("Certificate not specified (--cert)");
            process::exit(1);
          }
          if key_file == None {
            println!("Key not specified (--key)");
            process::exit(1);
          }
          builder.set_private_key_file(key_file.unwrap(), SslFiletype::PEM).unwrap();
          builder.set_certificate_chain_file(cert_file.unwrap()).unwrap();
        }

        srv = new_srv.bind_openssl("0.0.0.0:3095",builder)
        .unwrap()
        .run();
    } else {
        srv = new_srv.bind("0.0.0.0:3095")
        .unwrap()
        .run();
    }
    let srv_handle = srv.handle();
//                            service(fs::Files::new("/", "/var/netracer").index_file("index.html"))
   
    rt::spawn(srv);
    srv_handle.resume().await;

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let interface = args.interface;
    let max_dst_udp_port = args.max_dst_udp_port; //.unwrap_or(65535);
    let min_src_udp_port = args.min_src_udp_port; //.unwrap_or(2048);
    // Open the capture for the given interface
    let mut cap = pcap::Capture::from_device(interface.as_str()).unwrap()
        .promisc(true)  // Set the capture mode to promiscuous
        .snaplen(32000)  // Set the maximum bytes to capture per packet
        .immediate_mode(true)
        .open().unwrap();

    // Start capturing packets
    while let Ok(packet) = cap.next() {
//        println!("Received packet with length: {}", packet.header.len);
        // Here, you can add more processing or filtering logic if needed
        if let Some(ethernet_packet) = EthernetPacket::new(&packet.data) {
          match ethernet_packet.get_ethertype() {
             EtherTypes::Ipv4 => {
                  if let Some(ip_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                  match ip_packet.get_next_level_protocol() {
                    IpNextHeaderProtocols::Tcp => {
                      // Handle TCP packets
                      let tcp_packet = TcpPacket::new(ip_packet.payload());
                      if let Some(tcp_packet) = tcp_packet {
                         if (tcp_packet.get_flags() & pnet::packet::tcp::TcpFlags::SYN) != 0  && 
                            (tcp_packet.get_flags() & pnet::packet::tcp::TcpFlags::ACK) == 0 {
                            //println!("{:?}", tcp_packet.get_flags());
                      /*
                            println!(
                              "TCP Packet: {}:{} > {}:{}; Seq: {}, Ack: {}",
                              ip_packet.get_source(),
                              tcp_packet.get_source(),
                              ip_packet.get_destination(),
                              tcp_packet.get_destination(),
                              tcp_packet.get_sequence(),
                              tcp_packet.get_acknowledgement()
                            );
*/
                            let con0 = data::Connection {
                              addr : data::Address {
                                src: ip_packet.get_source().to_string(),
                                dst: ip_packet.get_destination().to_string(),
                                protocol: "TCP".to_string(),
                                port: tcp_packet.get_destination().to_string()
                              },
                              time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
                              max_speed: 0,
                              avg_speed: 0
                            };
                              let r = DB.r_transaction()?;
                              let con: Result<Option<data::Connection>, _> = r.get().primary(con0.clone().addr);
                              match con {
                                Ok(v) => match v {
                                  None => { 
                                    println!("New TCP connection {}:{} > {}:{}",
                                       ip_packet.get_source(),
                                       tcp_packet.get_source(),
                                       ip_packet.get_destination(),
                                       tcp_packet.get_destination()
                                    );
                                    let rw = DB.rw_transaction()?;
                                    // It's a good practice to use the latest version in your application
                                    rw.insert(con0)?;
                                    rw.commit()?;                                  
                                  },
                                  _ => { }
                                }
                                Err(e) => println!("{}",e)
                              }
                         }	                                          
                      }
                    },
                    IpNextHeaderProtocols::Udp => {
                      // Handle UDP packets
                      let udp_packet = UdpPacket::new(ip_packet.payload());
                      if let Some(udp_packet) = udp_packet {
/*
                          println!(
                              "UDP Packet: {}:{} > {}:{}; Len: {}",
                              ip_packet.get_source(),
                              udp_packet.get_source(),
                              ip_packet.get_destination(),
                              udp_packet.get_destination(),
                              udp_packet.get_length()
                          );
*/
                          let src_port = udp_packet.get_source();
                          let dst_port = udp_packet.get_destination();
                          if dst_port <=  max_dst_udp_port && src_port >= min_src_udp_port
                          { 
                            let con0 = data::Connection {
                              addr : data::Address {
                                src: ip_packet.get_source().to_string(),
                                dst: ip_packet.get_destination().to_string(),
                                protocol: "UDP".to_string(),
                                port: udp_packet.get_destination().to_string()
                              },
                              time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
                              max_speed: 0,
                              avg_speed: 0
                            };
                              let r = DB.r_transaction()?;
                              let con: Result<Option<data::Connection>, _> = r.get().primary(con0.clone().addr);
                              match con {
                                Ok(v) => match v {
                                  None => { 
                                    println!("New UDP connection {}:{} > {}:{}",
                                       ip_packet.get_source(),
                                       udp_packet.get_source(),
                                       ip_packet.get_destination(),
                                       udp_packet.get_destination()
                                    );
                                    let rw = DB.rw_transaction()?;
                                    // It's a good practice to use the latest version in your application
                                    rw.insert(con0)?;
                                    rw.commit()?;                                  
                                  },
                                  _ => { }
                                }
                                Err(e) => println!("{}",e)
                              }
                           }
                      }
                    },
                    _ => {
//                       println!("{:?}", ip_packet);
                    }
                  }
               }                
             },
             _ => {
//                       println!("{:?}", ethernet_packet);
             }
          }
        }
    }
  Ok(())
}
