import Postman from 'modules/postman';
import { Connection } from'./types';

export default class Connections {
  static all = () => Postman.get<Array<Connection>>('/conagg');
}
