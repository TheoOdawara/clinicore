import type { SessionClientKind } from "../../../common/decorators/session-client.decorator";

export enum SessionClient {
  Web = "web",
  Mobile = "mobile",
}

export const SESSION_CLIENT_BY_KIND: Record<SessionClientKind, SessionClient> =
  {
    web: SessionClient.Web,
    mobile: SessionClient.Mobile,
  };
