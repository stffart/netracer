import Axios, { AxiosRequestConfig } from 'axios';
//   baseURL: 'http://192.168.2.238:12347/',

const AxiosInstance = Axios.create({
  baseURL: '/',
});

export default class Postman {
  // TS Operator overloading example
  static get<Response>(ep: string): Promise<Response>;
  static get<Response>(ep: string, config: AxiosRequestConfig): Promise<Response>;

  static async get<Response>(ep: string, config?: AxiosRequestConfig) {
    return (await AxiosInstance.get<Response>(ep, config)).data;
  }

  static post = async <Data, Response>(ep: string, data: Data) => await AxiosInstance.post<Data, Response>(ep, data);
}
