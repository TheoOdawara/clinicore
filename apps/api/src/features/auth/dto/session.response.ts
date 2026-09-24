import { ApiProperty, ApiPropertyOptional } from "@nestjs/swagger";

export class SessionUserResponse {
  @ApiProperty({ format: "uuid" })
  id!: string;

  @ApiProperty({ example: "Ana Souza" })
  name!: string;

  @ApiProperty({ format: "email", example: "ana@exemplo.com" })
  email!: string;

  @ApiProperty()
  emailVerified!: boolean;

  @ApiProperty({ nullable: true, type: String })
  image!: string | null;
}

export class SessionTokensResponse {
  @ApiProperty({ description: "Sent as Authorization: Bearer" })
  accessToken!: string;

  @ApiProperty({ description: "<session uuid>.<43 base64url characters>" })
  refreshToken!: string;

  @ApiProperty({ example: 900 })
  accessTokenExpiresIn!: number;
}

export class SessionResponse {
  @ApiProperty({ type: SessionUserResponse })
  user!: SessionUserResponse;

  @ApiPropertyOptional({
    type: SessionTokensResponse,
    description: "Only with Clinicore-Client: mobile, which gets no cookie",
  })
  tokens?: SessionTokensResponse;
}

export class RefreshResponse {
  @ApiProperty({ type: SessionTokensResponse })
  tokens!: SessionTokensResponse;
}
