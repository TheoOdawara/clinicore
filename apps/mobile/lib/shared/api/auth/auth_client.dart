// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:dio/dio.dart';
import 'package:retrofit/retrofit.dart';

import '../models/clinicore_client.dart';
import '../models/email_dto.dart';
import '../models/password_reset_confirmation_dto.dart';
import '../models/refresh_response.dart';
import '../models/refresh_token_dto.dart';
import '../models/session_response.dart';
import '../models/sign_in_dto.dart';
import '../models/sign_up_dto.dart';
import '../models/token_dto.dart';

part 'auth_client.g.dart';

@RestApi()
abstract class AuthClient {
  factory AuthClient(Dio dio, {String? baseUrl}) = _AuthClient;

  /// Register an email and password account.
  ///
  /// Always answers 202 with an empty body, whether the email is new or already registered.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT.
  @POST('/users')
  Future<void> authControllerSignUp({
    @Body() required SignUpDto body,
    @Header('clinicore-client') ClinicoreClient? clinicoreClient,
  });

  /// Open a session.
  ///
  /// Issues the session cookies, or the tokens in the body with Clinicore-Client: mobile.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT.
  @POST('/sessions')
  Future<SessionResponse> authControllerSignIn({
    @Body() required SignInDto body,
    @Header('clinicore-client') ClinicoreClient? clinicoreClient,
  });

  /// Rotate the refresh token.
  ///
  /// Authenticated by the clinicore_refresh cookie, or by refreshToken in the body with Clinicore-Client: mobile. Presenting a token that was already rotated drops the whole session.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT.
  @POST('/sessions/current/tokens')
  Future<RefreshResponse> authControllerRefresh({
    @Body() required RefreshTokenDto body,
    @Header('clinicore-client') ClinicoreClient? clinicoreClient,
  });

  /// Close the current session.
  ///
  /// Deletes the session row and denies the access token that is still inside its 15 minutes.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT.
  @DELETE('/sessions/current')
  Future<void> authControllerSignOut({
    @Header('clinicore-client') ClinicoreClient? clinicoreClient,
  });

  /// Read the signed-in user.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT.
  @GET('/sessions/current')
  Future<SessionResponse> authControllerSession({
    @Header('clinicore-client') ClinicoreClient? clinicoreClient,
  });

  /// Send a new email verification link.
  ///
  /// Always answers 202 with an empty body. At most one link per address every 60 seconds and five every 24 hours.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT.
  @POST('/email-verifications')
  Future<void> authControllerRequestEmailVerification({
    @Body() required EmailDto body,
    @Header('clinicore-client') ClinicoreClient? clinicoreClient,
  });

  /// Confirm the email and open a session.
  ///
  /// Consumes every pending verification token of the address and issues the session cookies.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT.
  @POST('/email-verifications/confirmation')
  Future<void> authControllerConfirmEmailVerification({
    @Body() required TokenDto body,
    @Header('clinicore-client') ClinicoreClient? clinicoreClient,
  });

  /// Send a password reset link.
  ///
  /// Always answers 202 with an empty body, whether the address has an account or not. At most one link per address every 60 seconds and five every 24 hours.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT.
  @POST('/password-resets')
  Future<void> authControllerRequestPasswordReset({
    @Body() required EmailDto body,
    @Header('clinicore-client') ClinicoreClient? clinicoreClient,
  });

  /// Set a new password with the emailed token.
  ///
  /// Replaces the password, creating the password login for a Google-only account, and drops every session of the user at once.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 INVALID_CLIENT.
  @POST('/password-resets/confirmation')
  Future<void> authControllerConfirmPasswordReset({
    @Body() required PasswordResetConfirmationDto body,
    @Header('clinicore-client') ClinicoreClient? clinicoreClient,
  });
}
