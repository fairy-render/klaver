import { Url } from "@klaver/http";

const url = new Url("http://sr");

url.password = "test";
url;

console.log(url.href);
