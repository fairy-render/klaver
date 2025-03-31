import { Router, serve } from "@klaver/http";

const router = new Router();

router.get("/", () => {
	return new Response("Hello, World!");
});

router.get("/*rest", () => {
	return new Response("Rest");
});

serve(router);
