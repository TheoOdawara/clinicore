import { Injectable } from "@nestjs/common";
import { BusinessError } from "../../../common/exceptions/business-error";
import { SessionUserResponse } from "../dto/session.response";
import type { User } from "../entities/user.entity";
import { UserRepository } from "../repository/user.repository";

export function toSessionUser(user: User): SessionUserResponse {
  return {
    id: user.id,
    name: user.name,
    email: user.email,
    emailVerified: user.emailVerified,
    image: user.image,
  };
}

@Injectable()
export class SessionService {
  constructor(private readonly users: UserRepository) {}

  async currentUser(userId: string): Promise<SessionUserResponse> {
    const user = await this.users.findById(userId);

    if (user === null) {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    return toSessionUser(user);
  }
}
