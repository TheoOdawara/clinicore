import { ApiPropertyOptional } from "@nestjs/swagger";
import { IsOptional, Matches } from "class-validator";
import { REFRESH_TOKEN_FORMAT } from "../utils/session-token";

export class RefreshTokenDto {
  @ApiPropertyOptional({
    description:
      "Required with Clinicore-Client: mobile, ignored without it: <session uuid>.<43 base64url characters>",
    pattern: REFRESH_TOKEN_FORMAT.source,
  })
  @IsOptional()
  @Matches(REFRESH_TOKEN_FORMAT)
  refreshToken?: string;
}
