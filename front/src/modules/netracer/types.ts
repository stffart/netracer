export interface Address {
  src: string,
  dst: string,
  protocol: string,
  port: string
}

export interface Connection {
   time: number,
   max_speed: number,
   avg_speed: number,
   addr: Address
}

