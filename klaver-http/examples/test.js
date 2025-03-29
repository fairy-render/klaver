import { serve } from "@klaver/http";

console.log("HER!");

await serve((req) => new Response("Hello, World"));
