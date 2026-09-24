// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'session_user_response.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SessionUserResponse _$SessionUserResponseFromJson(Map<String, dynamic> json) =>
    SessionUserResponse(
      id: json['id'] as String,
      name: json['name'] as String,
      email: json['email'] as String,
      emailVerified: json['emailVerified'] as bool,
      image: json['image'] as String?,
    );

Map<String, dynamic> _$SessionUserResponseToJson(
  SessionUserResponse instance,
) => <String, dynamic>{
  'id': instance.id,
  'name': instance.name,
  'email': instance.email,
  'emailVerified': instance.emailVerified,
  'image': instance.image,
};
