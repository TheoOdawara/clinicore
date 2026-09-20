import {
  Global,
  Module,
  type MiddlewareConsumer,
  type NestModule,
} from "@nestjs/common";
import { EnvironmentService } from "../config/environment.service";
import { destinationFor } from "./destination";
import { LOGGER, createLogger } from "./logger";
import { PinoLoggerService } from "./pino-logger.service";
import { RequestLogMiddleware } from "./request-log.middleware";

@Global()
@Module({
  providers: [
    {
      provide: LOGGER,
      inject: [EnvironmentService],
      useFactory: (environment: EnvironmentService) =>
        createLogger(
          environment.get("LOG_LEVEL"),
          destinationFor(environment.get("NODE_ENV")),
        ),
    },
    PinoLoggerService,
    RequestLogMiddleware,
  ],
  exports: [LOGGER, PinoLoggerService],
})
export class LoggerModule implements NestModule {
  configure(consumer: MiddlewareConsumer): void {
    consumer.apply(RequestLogMiddleware).forRoutes("*splat");
  }
}
