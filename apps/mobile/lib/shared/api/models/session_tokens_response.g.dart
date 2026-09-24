// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'session_tokens_response.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SessionTokensResponse _$SessionTokensResponseFromJson(
  Map<String, dynamic> json,
) => SessionTokensResponse(
  accessToken: json['accessToken'] as String,
  refreshToken: json['refreshToken'] as String,
  accessTokenExpiresIn: json['accessTokenExpiresIn'] as num,
);

Map<String, dynamic> _$SessionTokensResponseToJson(
  SessionTokensResponse instance,
) => <String, dynamic>{
  'accessToken': instance.accessToken,
  'refreshToken': instance.refreshToken,
  'accessTokenExpiresIn': instance.accessTokenExpiresIn,
};
