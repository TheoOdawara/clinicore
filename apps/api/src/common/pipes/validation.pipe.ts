import { ValidationPipe } from "@nestjs/common";
import type { ValidationError } from "class-validator";
import { BusinessError, type ErrorFields } from "../exceptions/business-error";

const CAMEL_CASE_BOUNDARY = /([a-z0-9])([A-Z])/g;

function toConstraintCode(constraint: string): string {
  return constraint.replace(CAMEL_CASE_BOUNDARY, "$1_$2").toUpperCase();
}

function toFields(errors: ValidationError[], prefix = ""): ErrorFields {
  const fields: ErrorFields = {};
  for (const error of errors) {
    const path = `${prefix}${error.property}`;
    Object.assign(fields, toFields(error.children ?? [], `${path}.`));
    const violated = Object.keys(error.constraints ?? {})[0];
    if (violated === undefined) {
      continue;
    }
    fields[path] = toConstraintCode(violated);
  }
  return fields;
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
        toFields(errors),
      ),
  });
}
