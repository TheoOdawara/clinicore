export type ErrorCode =
  | "VALIDATION_FAILED"
  | "INVALID_TOKEN"
  | "INVALID_PASSWORD"
  | "INVALID_CREDENTIALS"
  | "INVALID_SESSION"
  | "SESSION_REUSED"
  | "EMAIL_NOT_VERIFIED"
  | "INVALID_ORIGIN"
  | "RATE_LIMITED"
  | "SERVICE_UNAVAILABLE";

export type BusinessErrorType =
  | "NotFound"
  | "Conflict"
  | "Forbidden"
  | "Invalid"
  | "Unauthorized"
  | "RateLimited"
  | "Unavailable";

export interface FieldError {
  pointer: string;
  code: string;
}

export class BusinessError extends Error {
  readonly type: BusinessErrorType;
  readonly code: ErrorCode;
  readonly errors: FieldError[];

  private constructor(
    type: BusinessErrorType,
    code: ErrorCode,
    message: string,
    errors: FieldError[],
  ) {
    super(message);
    this.name = "BusinessError";
    this.type = type;
    this.code = code;
    this.errors = errors;
  }

  static notFound(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("NotFound", code, message, []);
  }

  static conflict(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("Conflict", code, message, []);
  }

  static forbidden(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("Forbidden", code, message, []);
  }

  static invalid(
    code: ErrorCode,
    message: string,
    errors: FieldError[] = [],
  ): BusinessError {
    return new BusinessError("Invalid", code, message, errors);
  }

  static unauthorized(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("Unauthorized", code, message, []);
  }

  static rateLimited(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("RateLimited", code, message, []);
  }

  static unavailable(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("Unavailable", code, message, []);
  }
}
