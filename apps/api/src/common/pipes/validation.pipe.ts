import { ValidationPipe } from "@nestjs/common";
import type { ValidationError } from "class-validator";
import { BusinessError, type FieldError } from "../exceptions/business-error";

const CAMEL_CASE_BOUNDARY = /([a-z0-9])([A-Z])/g;

function toConstraintCode(constraint: string): string {
  return constraint.replace(CAMEL_CASE_BOUNDARY, "$1_$2").toUpperCase();
}

function toFieldErrors(errors: ValidationError[], pointer = "#"): FieldError[] {
  return errors.flatMap((error) => {
    const path = `${pointer}/${error.property}`;
    const nested = toFieldErrors(error.children ?? [], path);
    const violated = Object.keys(error.constraints ?? {})[0];
    if (violated === undefined) {
      return nested;
    }
    return [...nested, { pointer: path, code: toConstraintCode(violated) }];
  });
}

export function validationPipe(): ValidationPipe {
  return new ValidationPipe({
    whitelist: true,
    forbidNonWhitelisted: true,
    transform: true,
    exceptionFactory: (errors: ValidationError[]) =>
      BusinessError.invalid(
        "VALIDATION_FAILED",
        "Validation failed",
        toFieldErrors(errors),
      ),
  });
}
