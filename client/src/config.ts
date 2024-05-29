export const RUSTY_BASE_URL = import.meta.env.PROD ?
  'https://mt5baplmo6.execute-api.us-east-1.amazonaws.com/Prod' :
  'http://localhost:9000/lambda-url/lambda-handler';

export const ORIGIN = import.meta.env.PROD ?
  'https://binhrobles.com' :
  'http://localhost:5173';
