import { ApiProperty } from "@nestjs/swagger";
import { Transform } from "class-transformer";
import { IsEmail, IsString, Length } from "class-validator";
import { IsStrongPassword } from "./is-strong-password.decorator";

function trimmed({ value }: { value: unknown }): unknown {
  if (typeof value !== "string") {
    return value;
  }

  return value.trim();
}

export class SignUpDto {
  @Transform(trimmed)
  @IsString()
  @Length(1, 100)
  name!: string;

  @IsEmail()
  @Length(1, 320)
  email!: string;

  @ApiProperty({
    minLength: 8,
    maxLength: 128,
    description:
      "At least 8 and at most 128 characters, with an uppercase letter, a digit and a character that is neither a letter nor a digit",
    example: "Clinica#2026",
  })
  @IsStrongPassword()
  password!: string;
}
