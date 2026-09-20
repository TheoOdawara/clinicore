import { createServer } from "node:net";

export function freePort(): Promise<number> {
  const probe = createServer();

  return new Promise((settle, fail) => {
    probe.on("error", fail);
    probe.listen(0, "127.0.0.1", () => {
      const address = probe.address();
      if (address === null || typeof address === "string") {
        fail(new Error("the probe server reported no port"));
        return;
      }
      probe.close(() => {
        settle(address.port);
      });
    });
  });
}
