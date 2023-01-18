import puppeteer from "https://deno.land/x/puppeteer@16.2.0/mod.ts";
import { Base64Url } from "https://deno.land/x/base64url@v0.0.3/mod.ts";

const prometheusUrl = new URL(
  Deno.env.get("PROMETHEUS_URL") || "http://localhost:9090"
);
const port = parseInt(Deno.env.get("PORT") || "8080");
const server = Deno.listen({ port });
console.log(`Listening on ${port}`);
const browser = await puppeteer.launch({
  defaultViewport: {
    width: 1040,
    height: 10400,
  },
});
const base64url = new Base64Url();

for await (const conn of server) {
  handleHttp(conn);
}
await browser.close();

async function handleHttp(conn: Deno.Conn) {
  const httpConn = Deno.serveHttp(conn);
  for await (const requestEvent of httpConn) {
    try {
      const requestUrl = new URL(requestEvent.request.url);

      // Route the request
      let response;
      if (requestUrl.pathname.startsWith("/render")) {
        response = await prometheusSsr(requestUrl);
      } else {
        response = prometheusRedirect(requestUrl);
      }
      await requestEvent.respondWith(response);
    } catch (err) {
      console.error(err);
      try {
        await requestEvent.respondWith(
          new Response(null, {
            status: 500,
          })
        );
      } catch (_err) {
        // Ignore error trying to return error
      }
    }
  }
}

async function prometheusSsr(requestUrl: URL): Promise<Response> {
  const encodedUrl = requestUrl.searchParams.get("url");
  if (!encodedUrl) {
    return new Response("Missing `url` query parameter", {
      status: 400,
    });
  }

  const decodedUrl = new URL(base64url.decode(encodedUrl));
  const url = new URL(prometheusUrl);
  url.pathname = decodedUrl.pathname;
  url.search = decodedUrl.search;

  console.log(`Loading ${url}`);

  const page = await browser.newPage();
  await page.goto(url.toString(), { waitUntil: "domcontentloaded" });

  // Wait for the graph to render
  try {
    await page.waitForSelector("canvas", { timeout: 1000 });
  } catch (_err) {
    // If there was no canvas, it probably means there were no query results
    return new Response(null, {
      // No content
      status: 204,
      headers: [
        ["Cache-Control", "max-age=5"],
        ["Content-Type", "image/png"],
      ],
    });
  }

  console.log("Loaded");
  const graph = await page.$(".tab-content");
  const screenshot = await graph?.screenshot({
    type: "png",
    omitBackground: true,
  });

  return new Response(screenshot, {
    status: 200,
    headers: [
      ["Cache-Control", "max-age=5"],
      ["Content-Type", "image/png"],
    ],
  });
}

function prometheusRedirect(requestUrl: URL): Response {
  const url = new URL(prometheusUrl);
  url.pathname = requestUrl.pathname;
  url.search = requestUrl.search;

  return new Response(null, {
    status: 301,
    headers: [["Location", url.toString()]],
  });
}
