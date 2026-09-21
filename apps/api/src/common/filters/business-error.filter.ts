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

const STATUS_BY_TYPE: Record<BusinessErrorType, HttpStatus> = {
  Invalid: HttpStatus.BAD_REQUEST,
  Unauthorized: HttpStatus.UNAUTHORIZED,
  Forbidden: HttpStatus.FORBIDDEN,
  NotFound: HttpStatus.NOT_FOUND,
  Conflict: HttpStatus.CONFLICT,
  RateLimited: HttpStatus.TOO_MANY_REQUESTS,
  Unavailable: HttpStatus.SERVICE_UNAVAILABLE,
};

const INTERNAL_SERVER_ERROR: number = HttpStatus.INTERNAL_SERVER_ERROR;
const PROBLEM_CONTENT_TYPE = "application/problem+json";
const PROBLEM_TYPE_PREFIX = "tag:clinicore.com.br,2026:";

interface Problem {
  type: string;
  title: string;
  status: number;
  errors?: FieldError[];
}

function problemTypeOf(code: ErrorCode): string {
  return `${PROBLEM_TYPE_PREFIX}${code.toLowerCase().replaceAll("_", "-")}`;
}

function businessProblem(error: BusinessError): Problem {
  const problem: Problem = {
    type: problemTypeOf(error.code),
    title: error.message,
    status: STATUS_BY_TYPE[error.type],
  };
  if (error.errors.length > 0) {
    problem.errors = error.errors;
  }
  return problem;
}

function blankProblem(status: number): Problem {
  return {
    type: "about:blank",
    title: STATUS_CODES[status] ?? "Unknown Error",
    status,
  };
}

@Catch()
export class BusinessErrorFilter implements ExceptionFilter {
  constructor(@Inject(LOGGER) private readonly logger: Logger) {}

  catch(exception: unknown, host: ArgumentsHost): void {
    const response = host.switchToHttp().getResponse<Response>();

    if (exception instanceof BusinessError) {
      this.send(response, businessProblem(exception));
      return;
    }

    let status = INTERNAL_SERVER_ERROR;
    if (exception instanceof HttpException) {
      status = exception.getStatus();
    }
    if (status >= INTERNAL_SERVER_ERROR) {
      this.logger.error({ err: exception }, "unhandled error");
    }

    this.send(response, blankProblem(status));
  }

  private send(response: Response, problem: Problem): void {
    response.status(problem.status).type(PROBLEM_CONTENT_TYPE).json(problem);
  }
}
