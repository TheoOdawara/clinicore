import { Injectable } from "@nestjs/common";
import { hash } from "@node-rs/argon2";
import { UserRepository } from "../repository/user.repository";
import { issueToken } from "../utils/verification-token";
import { EmailVerificationService } from "./email-verification.service";

@Injectable()
export class SignUpService {
  constructor(
    private readonly users: UserRepository,
    private readonly emailVerification: EmailVerificationService,
  ) {}

  async signUp(name: string, email: string, password: string): Promise<void> {
    const address = email.toLowerCase();
    const passwordHash = await hash(password);
    const issued = issueToken();

    const outcome = await this.users.createWithCredentialAccount(
      name,
      address,
      passwordHash,
      issued.token,
    );

    if (outcome === "created-with-token") {
      this.emailVerification.sendLink(address, name, issued.secret);
    }
  }
}
