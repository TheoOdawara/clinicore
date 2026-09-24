// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'sign_up_dto.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SignUpDto _$SignUpDtoFromJson(Map<String, dynamic> json) => SignUpDto(
  password: json['password'] as String,
  name: json['name'] as String,
  email: json['email'] as String,
);

Map<String, dynamic> _$SignUpDtoToJson(SignUpDto instance) => <String, dynamic>{
  'password': instance.password,
  'name': instance.name,
  'email': instance.email,
};
