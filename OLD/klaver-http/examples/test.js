import { Router, serve } from "@klaver/http";

const router = new Router();

router.get("/", () => {
	console.log("root");
	return new Response("Hello, World!");
});

router.get("/*rest", () => {
	return new Response("Rest");
});

router.use(async (req, next) => {
	console.log("before");
	const out = await next.call(req);
	console.log("after");
	return out;
});

serve(router);
