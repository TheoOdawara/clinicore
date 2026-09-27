import {
  Catch,
  HttpException,
  HttpStatus,
  Inject,
  type ArgumentsHost,
  type ExceptionFilter,
} from "@nestjs/common";
import type { Response } from "express";
import { STATUS_CODES } from "node:http";
import type { Logger } from "pino";
import { LOGGER } from "../../core/logger/logger";
import {
  BusinessError,
  type BusinessErrorType,
  type ErrorCode,
  type FieldError,
} from "../exceptions/business-error";

interface Problem {
  type: string;
  title: string;
  status: number;
  errors?: FieldError[];
}

@Catch()
export class BusinessErrorFilter implements ExceptionFilter {
  constructor(@Inject(LOGGER) private readonly logger: Logger) {}

  catch(exception: unknown, host: ArgumentsHost): void {
    const statusByType: Record<BusinessErrorType, HttpStatus> = {
      Invalid: HttpStatus.BAD_REQUEST,
      Unauthorized: HttpStatus.UNAUTHORIZED,
      Forbidden: HttpStatus.FORBIDDEN,
      NotFound: HttpStatus.NOT_FOUND,
      Conflict: HttpStatus.CONFLICT,
      RateLimited: HttpStatus.TOO_MANY_REQUESTS,
      Unavailable: HttpStatus.SERVICE_UNAVAILABLE,
    };
    const internalServerError: number = HttpStatus.INTERNAL_SERVER_ERROR;
    const problemTypeOf = (code: ErrorCode): string =>
      `tag:clinicore.com.br,2026:${code.toLowerCase().replaceAll("_", "-")}`;
    const businessProblem = (error: BusinessError): Problem => {
      const problem: Problem = {
        type: problemTypeOf(error.code),
        title: error.message,
        status: statusByType[error.type],
      };
      if (error.errors.length > 0) {
        problem.errors = error.errors;
      }
      return problem;
    };
    const blankProblem = (status: number): Problem => ({
      type: "about:blank",
      title: STATUS_CODES[status] ?? "Unknown Error",
      status,
    });
    const response = host.switchToHttp().getResponse<Response>();

    if (exception instanceof BusinessError) {
      this.send(response, businessProblem(exception));
      return;
    }

    let status = internalServerError;
    if (exception instanceof HttpException) {
      status = exception.getStatus();
    }
    if (status >= internalServerError) {
      this.logger.error({ err: exception }, "unhandled error");
    }

    this.send(response, blankProblem(status));
  }

  private send(response: Response, problem: Problem): void {
    response
      .status(problem.status)
      .type("application/problem+json")
      .json(problem);
  }
}
