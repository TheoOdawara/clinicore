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
  | "INTERNAL_ERROR"
  | "SERVICE_UNAVAILABLE";

export type BusinessErrorType =
  "NotFound" | "Conflict" | "Forbidden" | "Invalid" | "Unauthorized";

export type ErrorFields = Record<string, string>;

export class BusinessError extends Error {
  readonly type: BusinessErrorType;
  readonly code: ErrorCode;
  readonly fields: ErrorFields;

  private constructor(
    type: BusinessErrorType,
    code: ErrorCode,
    message: string,
    fields: ErrorFields,
  ) {
    super(message);
    this.name = "BusinessError";
    this.type = type;
    this.code = code;
    this.fields = fields;
  }

  static notFound(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("NotFound", code, message, {});
  }

  static conflict(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("Conflict", code, message, {});
  }

  static forbidden(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("Forbidden", code, message, {});
  }

  static invalid(
    code: ErrorCode,
    message: string,
    fields: ErrorFields = {},
  ): BusinessError {
    return new BusinessError("Invalid", code, message, fields);
  }

  static unauthorized(code: ErrorCode, message: string): BusinessError {
    return new BusinessError("Unauthorized", code, message, {});
  }
}
