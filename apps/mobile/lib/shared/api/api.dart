// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:dio/dio.dart';
import 'package:json_annotation/json_annotation.dart';
import 'package:retrofit/retrofit.dart';

part 'api.g.dart';

@RestApi()
abstract class AuthClient {
  factory AuthClient(Dio dio, {String? baseUrl}) = _AuthClient;

  /// Send a new email verification link.
  ///
  /// Always answers 202 with an empty body. At most one link per address every 60 seconds and five every 24 hours.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 invalid-client.
  @POST('/email-verifications')
  Future<void> requestEmailVerification({
    @Body() required EmailVerificationRequest body,
    @Header('Clinicore-Client') ClinicoreClient? clinicoreClient,
  });

  /// Confirm the email.
  ///
  /// Marks the email as verified and opens no session: the owner signs in afterwards with the password they chose.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 invalid-client.
  @POST('/email-verifications/confirmation')
  Future<void> confirmEmail({
    @Body() required EmailConfirmationRequest body,
    @Header('Clinicore-Client') ClinicoreClient? clinicoreClient,
  });

  /// Sign in with email and password.
  ///
  /// The web gets the session in two cookies. With Clinicore-Client: mobile the tokens come in the body and no cookie is set.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 invalid-client.
  @POST('/sessions')
  Future<SignInResponse> signIn({
    @Body() required SignInRequest body,
    @Header('Clinicore-Client') ClinicoreClient? clinicoreClient,
  });

  /// Read the signed-in user.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 invalid-client.
  @GET('/sessions/current')
  Future<SessionResponse> readCurrentSession({
    @Header('Clinicore-Client') ClinicoreClient? clinicoreClient,
  });

  /// Sign out and revoke the session at once.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 invalid-client.
  @DELETE('/sessions/current')
  Future<void> signOut({
    @Header('Clinicore-Client') ClinicoreClient? clinicoreClient,
  });

  /// Rotate the refresh token.
  ///
  /// The web sends the refresh cookie and no body, and gets 204 with new cookies. With Clinicore-Client: mobile the refresh token goes in the body and the new tokens come back in it.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 invalid-client.
  @POST('/sessions/current/tokens')
  Future<TokenRefreshResponse> refresh({
    @Header('Clinicore-Client') ClinicoreClient? clinicoreClient,
    @Body() TokenRefreshRequest? body,
  });

  /// Register an email and password account.
  ///
  /// Always answers 202 with an empty body, whether the email is new or already registered.
  ///
  /// [clinicoreClient] - mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 invalid-client.
  @POST('/users')
  Future<void> signUp({
    @Body() required SignUpRequest body,
    @Header('Clinicore-Client') ClinicoreClient? clinicoreClient,
  });
}

@RestApi()
abstract class HealthClient {
  factory HealthClient(Dio dio, {String? baseUrl}) = _HealthClient;

  @GET('/health')
  Future<Health> check();
}

@JsonSerializable()
class EmailConfirmationRequest {
  const EmailConfirmationRequest({required this.token});

  factory EmailConfirmationRequest.fromJson(Map<String, Object?> json) =>
      _$EmailConfirmationRequestFromJson(json);

  final String token;

  Map<String, Object?> toJson() => _$EmailConfirmationRequestToJson(this);
}

@JsonSerializable()
class EmailVerificationRequest {
  const EmailVerificationRequest({required this.email});

  factory EmailVerificationRequest.fromJson(Map<String, Object?> json) =>
      _$EmailVerificationRequestFromJson(json);

  final String email;

  Map<String, Object?> toJson() => _$EmailVerificationRequestToJson(this);
}

@JsonSerializable()
class FieldError {
  const FieldError({required this.code, required this.pointer});

  factory FieldError.fromJson(Map<String, Object?> json) =>
      _$FieldErrorFromJson(json);

  final String code;
  final String pointer;

  Map<String, Object?> toJson() => _$FieldErrorToJson(this);
}

@JsonSerializable()
class Health {
  const Health({required this.status});

  factory Health.fromJson(Map<String, Object?> json) => _$HealthFromJson(json);

  final String status;

  Map<String, Object?> toJson() => _$HealthToJson(this);
}

@JsonSerializable()
class Problem {
  const Problem({
    required this.status,
    required this.title,
    required this.type,
    this.errors,
  });

  factory Problem.fromJson(Map<String, Object?> json) =>
      _$ProblemFromJson(json);

  final List<FieldError>? errors;
  final int status;
  final String title;
  final String type;

  Map<String, Object?> toJson() => _$ProblemToJson(this);
}

@JsonSerializable()
class SessionResponse {
  const SessionResponse({required this.user});

  factory SessionResponse.fromJson(Map<String, Object?> json) =>
      _$SessionResponseFromJson(json);

  final SessionUser user;

  Map<String, Object?> toJson() => _$SessionResponseToJson(this);
}

@JsonSerializable()
class SessionTokens {
  const SessionTokens({
    required this.accessToken,
    required this.accessTokenExpiresIn,
    required this.refreshToken,
  });

  factory SessionTokens.fromJson(Map<String, Object?> json) =>
      _$SessionTokensFromJson(json);

  final String accessToken;
  final int accessTokenExpiresIn;
  final String refreshToken;

  Map<String, Object?> toJson() => _$SessionTokensToJson(this);
}

@JsonSerializable()
class SessionUser {
  const SessionUser({
    required this.email,
    required this.emailVerified,
    required this.id,
    required this.name,
    this.image,
  });

  factory SessionUser.fromJson(Map<String, Object?> json) =>
      _$SessionUserFromJson(json);

  final String email;
  final bool emailVerified;
  final String id;
  final String? image;
  final String name;

  Map<String, Object?> toJson() => _$SessionUserToJson(this);
}

@JsonSerializable()
class SignInRequest {
  const SignInRequest({required this.email, required this.password});

  factory SignInRequest.fromJson(Map<String, Object?> json) =>
      _$SignInRequestFromJson(json);

  final String email;
  final String password;

  Map<String, Object?> toJson() => _$SignInRequestToJson(this);
}

@JsonSerializable()
class SignInResponse {
  const SignInResponse({required this.user, this.tokens});

  factory SignInResponse.fromJson(Map<String, Object?> json) =>
      _$SignInResponseFromJson(json);

  final SessionTokens? tokens;
  final SessionUser user;

  Map<String, Object?> toJson() => _$SignInResponseToJson(this);
}

@JsonSerializable()
class SignUpRequest {
  const SignUpRequest({
    required this.email,
    required this.name,
    required this.password,
  });

  factory SignUpRequest.fromJson(Map<String, Object?> json) =>
      _$SignUpRequestFromJson(json);

  final String email;
  final String name;
  final String password;

  Map<String, Object?> toJson() => _$SignUpRequestToJson(this);
}

@JsonSerializable()
class TokenRefreshRequest {
  const TokenRefreshRequest({required this.refreshToken});

  factory TokenRefreshRequest.fromJson(Map<String, Object?> json) =>
      _$TokenRefreshRequestFromJson(json);

  final String refreshToken;

  Map<String, Object?> toJson() => _$TokenRefreshRequestToJson(this);
}

@JsonSerializable()
class TokenRefreshResponse {
  const TokenRefreshResponse({required this.tokens});

  factory TokenRefreshResponse.fromJson(Map<String, Object?> json) =>
      _$TokenRefreshResponseFromJson(json);

  final SessionTokens tokens;

  Map<String, Object?> toJson() => _$TokenRefreshResponseToJson(this);
}

@JsonEnum()
enum ClinicoreClient {
  @JsonValue('mobile')
  mobile('mobile'),

  /// Default value for all unparsed values, allows backward compatibility when adding new values on the backend.
  $unknown(null);

  const ClinicoreClient(this.json);

  factory ClinicoreClient.fromJson(String json) =>
      values.firstWhere((e) => e.json == json, orElse: () => $unknown);

  final String? json;

  @override
  String toString() => json?.toString() ?? super.toString();

  /// Returns all defined enum values excluding the $unknown value.
  static List<ClinicoreClient> get $valuesDefined =>
      values.where((value) => value != $unknown).toList();
}

/// Clinicore API `v0.1.0`.
///
///
class RestClient {
  RestClient(Dio dio, {String? baseUrl}) : _dio = dio, _baseUrl = baseUrl;

  final Dio _dio;
  final String? _baseUrl;

  static String get version => '0.1.0';

  AuthClient? _auth;
  HealthClient? _health;

  AuthClient get auth => _auth ??= AuthClient(_dio, baseUrl: _baseUrl);

  HealthClient get health => _health ??= HealthClient(_dio, baseUrl: _baseUrl);
}
