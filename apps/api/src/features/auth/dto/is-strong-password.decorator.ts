import { registerDecorator } from "class-validator";
import { isStrongPassword } from "../utils/password-policy";

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
