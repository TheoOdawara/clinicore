// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'details.g.dart';

@JsonSerializable()
class Details {
  const Details({required this.status});

  factory Details.fromJson(Map<String, Object?> json) =>
      _$DetailsFromJson(json);

  final String status;

  Map<String, Object?> toJson() => _$DetailsToJson(this);
}
