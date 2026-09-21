import { Injectable } from "@nestjs/common";
import { hash } from "@node-rs/argon2";
import { UserRepository } from "../repository/user.repository";

@Injectable()
export class SignUpService {
  constructor(private readonly users: UserRepository) {}

  async signUp(name: string, email: string, password: string): Promise<void> {
    const passwordHash = await hash(password);

    await this.users.createWithCredentialAccount(
      name,
      email.toLowerCase(),
      passwordHash,
    );
  }
}
