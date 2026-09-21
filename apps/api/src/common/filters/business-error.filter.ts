import {
  Catch,
  HttpException,
  HttpStatus,
  Inject,
  type ArgumentsHost,
  type ExceptionFilter,
} from "@nestjs/common";
import type { Response } from "express";
import type { Logger } from "pino";
import { LOGGER } from "../../core/logger/logger";
import {
  BusinessError,
  type BusinessErrorType,
} from "../exceptions/business-error";

const STATUS_BY_TYPE: Record<BusinessErrorType, HttpStatus> = {
  Invalid: HttpStatus.BAD_REQUEST,
  Unauthorized: HttpStatus.UNAUTHORIZED,
  Forbidden: HttpStatus.FORBIDDEN,
  NotFound: HttpStatus.NOT_FOUND,
  Conflict: HttpStatus.CONFLICT,
  Unavailable: HttpStatus.SERVICE_UNAVAILABLE,
};

const INTERNAL_SERVER_ERROR: number = HttpStatus.INTERNAL_SERVER_ERROR;

@Catch()
export class BusinessErrorFilter implements ExceptionFilter {
  constructor(@Inject(LOGGER) private readonly logger: Logger) {}

  catch(exception: unknown, host: ArgumentsHost): void {
    const response = host.switchToHttp().getResponse<Response>();

    if (exception instanceof BusinessError) {
      response.status(STATUS_BY_TYPE[exception.type]).json({
        code: exception.code,
        message: exception.message,
        fields: exception.fields,
      });
      return;
    }

    if (
      exception instanceof HttpException &&
      exception.getStatus() >= INTERNAL_SERVER_ERROR
    ) {
      this.respondServerError(exception, exception.getStatus(), response);
      return;
    }

    if (exception instanceof HttpException) {
      const status = exception.getStatus();
      response.status(status).json({
        code: HttpStatus[status] ?? "INTERNAL_ERROR",
        message: exception.message,
        fields: {},
      });
      return;
    }

    this.respondServerError(exception, INTERNAL_SERVER_ERROR, response);
  }

  private respondServerError(
    exception: unknown,
    status: number,
    response: Response,
  ): void {
    this.logger.error({ err: exception }, "unhandled error");
    if (status === INTERNAL_SERVER_ERROR) {
      response.status(status).json({
        code: "INTERNAL_ERROR",
        message: "Internal server error",
        fields: {},
      });
      return;
    }
    response.status(status).json({
      code: HttpStatus[status] ?? "INTERNAL_ERROR",
      message: "Server error",
      fields: {},
    });
  }
}
