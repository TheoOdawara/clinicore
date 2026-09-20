import { createLogger } from "../logger";
import { PinoLoggerService } from "../pino-logger.service";

const STACK = "Error: connection refused\n    at connect (/app/db.ts:10:5)";

type LogLine = Record<string, unknown>;

function capture(emit: (service: PinoLoggerService) => void): LogLine {
  const written: string[] = [];
  const logger = createLogger("info", {
    write(line: string): void {
      written.push(line);
    },
  });

  emit(new PinoLoggerService(logger));

  return JSON.parse(written.join("")) as LogLine;
}

describe("PinoLoggerService.error", () => {
  it("keeps the stack and the context Nest passes", () => {
    const line = capture((service) => {
      service.error("Unable to connect", STACK, "TypeOrmModule");
    });

    expect(line).toMatchObject({
      msg: "Unable to connect",
      stack: STACK,
      context: "TypeOrmModule",
    });
  });

  it("reads a lone stack as the stack, never as the context", () => {
    const line = capture((service) => {
      service.error("Unable to connect", STACK);
    });

    expect(line).toMatchObject({ msg: "Unable to connect", stack: STACK });
    expect(line.context).toBeUndefined();
  });

  it("reads a lone plain string as the context", () => {
    const line = capture((service) => {
      service.error("Unable to connect", "TypeOrmModule");
    });

    expect(line).toMatchObject({ context: "TypeOrmModule" });
    expect(line.stack).toBeUndefined();
  });

  it("serializes an Error message with its stack", () => {
    const line = capture((service) => {
      service.error(new Error("probe failure"));
    });

    expect(line.msg).toBe("probe failure");
    expect(line.err).toMatchObject({
      message: "probe failure",
      stack: expect.stringContaining("pino-logger.service.test.ts") as unknown,
    });
  });
});
