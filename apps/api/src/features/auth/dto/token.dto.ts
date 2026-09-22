import { ApiProperty } from "@nestjs/swagger";
import { Matches } from "class-validator";

export const TOKEN_FORMAT = /^[A-Za-z0-9_-]{43}$/;

export class TokenDto {
  @ApiProperty({
    description: "The token from the emailed link: 43 base64url characters",
    pattern: TOKEN_FORMAT.source,
  })
  @Matches(TOKEN_FORMAT)
  token!: string;
}
