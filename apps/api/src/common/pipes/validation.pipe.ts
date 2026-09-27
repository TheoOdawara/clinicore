import { ValidationPipe } from "@nestjs/common";
import type { ValidationError } from "class-validator";
import { BusinessError, type FieldError } from "../exceptions/business-error";

export function validationPipe(): ValidationPipe {
  const toConstraintCode = (constraint: string): string =>
    constraint.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toUpperCase();
  const toFieldErrors = (
    errors: ValidationError[],
    pointer = "#",
  ): FieldError[] =>
    errors.flatMap((error) => {
      const path = `${pointer}/${error.property}`;
      const nested = toFieldErrors(error.children ?? [], path);
      const violated = Object.keys(error.constraints ?? {})[0];
      if (violated === undefined) {
        return nested;
      }
      return [...nested, { pointer: path, code: toConstraintCode(violated) }];
    });

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
