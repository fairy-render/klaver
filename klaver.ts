import { Client, Request } from "@klaver/http";

const client = new Client();

const resp = await client.send(new Request("https://github.com"));

console.log(await resp.text());
