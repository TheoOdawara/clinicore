import { ApiProperty } from "@nestjs/swagger";

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

export class SessionResponse {
  @ApiProperty({ type: SessionUserResponse })
  user!: SessionUserResponse;
}
