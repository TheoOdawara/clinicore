import { registerDecorator } from "class-validator";

export function isStrongPassword(password: string): boolean {
  if (password.length < 8 || password.length > 128) {
    return false;
  }

  const uppercase = /\p{Lu}/u;
  const digit = /\p{Nd}/u;
  const neitherLetterNorDigit = /[^\p{L}\p{Nd}]/u;

  return (
    uppercase.test(password) &&
    digit.test(password) &&
    neitherLetterNorDigit.test(password)
  );
}

export function IsStrongPassword() {
  return function decorate(target: object, propertyName: string): void {
    registerDecorator({
      name: "weakPassword",
      target: target.constructor,
      propertyName,
      validator: {
        validate: (value: unknown) =>
          typeof value === "string" && isStrongPassword(value),
      },
    });
  };
}
