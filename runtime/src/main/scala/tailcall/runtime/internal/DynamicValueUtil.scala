package tailcall.runtime.internal

import caliban.{InputValue, ResponseValue, Value}
import zio.Chunk
import zio.json.ast.Json
import zio.schema.{DynamicValue, Schema, StandardType, TypeId}

import java.math.{BigDecimal => BigDecimalJava}
import scala.collection.immutable.ListMap

import InputValue.{ListValue => InputList, ObjectValue => InputObject, VariableValue}
import ResponseValue.{ListValue => ResponseList, ObjectValue => ResponseObject, StreamValue}
import Value.FloatValue.{BigDecimalNumber, DoubleNumber, FloatNumber}
import Value.IntValue.{BigIntNumber, IntNumber, LongNumber}
import Value.{BooleanValue, EnumValue, NullValue, StringValue}

object DynamicValueUtil {
  def asString(dv: DynamicValue): Option[String] =
    dv match {
      case DynamicValue.Primitive(value, _) => Some(value.toString)
      case _                                => None
    }

  private def toValue[A](value: A, standardType: StandardType[A]): Value =
    standardType match {
      case StandardType.StringType         => Value.StringValue(value.toString)
      case StandardType.IntType            => Value.IntValue(value.toString.toInt)
      case StandardType.MonthDayType       => Value.StringValue(value.toString)
      case StandardType.LocalDateTimeType  => Value.StringValue(value.toString)
      case StandardType.BoolType           => Value.BooleanValue(value.toString.toBoolean)
      case StandardType.LocalTimeType      => Value.StringValue(value.toString)
      case StandardType.OffsetDateTimeType => Value.StringValue(value.toString)
      case StandardType.MonthType          => Value.StringValue(value.toString)
      case StandardType.ShortType          => Value.IntValue(value.toString.toShort)
      case StandardType.ZoneIdType         => Value.StringValue(value.toString)
      case StandardType.BigDecimalType     => Value.FloatValue(BigDecimal(value.toString))
      case StandardType.YearType           => Value.IntValue(value.toString.toInt)
      case StandardType.ByteType           => Value.IntValue(value.toString.toByte)
      case StandardType.UUIDType           => Value.StringValue(value.toString)
      case StandardType.PeriodType         => Value.StringValue(value.toString)
      case StandardType.LongType           => Value.IntValue(value.toString.toLong)
      case StandardType.ZoneOffsetType     => Value.StringValue(value.toString)
      case StandardType.BigIntegerType     => Value.IntValue(BigInt(value.toString))
      case StandardType.OffsetTimeType     => Value.StringValue(value.toString)
      case StandardType.UnitType           => Value.NullValue
      case StandardType.DoubleType         => Value.FloatValue(value.toString.toDouble)
      case StandardType.InstantType        => Value.StringValue(value.toString)
      case StandardType.FloatType          => Value.FloatValue(value.toString.toFloat)
      case StandardType.LocalDateType      => Value.StringValue(value.toString)
      case StandardType.ZonedDateTimeType  => Value.StringValue(value.toString)
      case StandardType.YearMonthType      => Value.StringValue(value.toString)
      case StandardType.CharType           => Value.StringValue(value.toString)
      case StandardType.BinaryType         => Value
          .StringValue(java.util.Base64.getEncoder.encodeToString(value.asInstanceOf[Chunk[Byte]].toArray))
      case StandardType.DurationType       => Value.StringValue(value.toString)
      case StandardType.DayOfWeekType      => Value.StringValue(value.toString)
    }

  def toResponseValue(input: DynamicValue): Option[ResponseValue] = {
    input match {
      case DynamicValue.Sequence(values) => Some(ResponseValue.ListValue(values.flatMap(toResponseValue).toList))
      case DynamicValue.Primitive(value, standardType) => Some(toValue(value, standardType))
      case DynamicValue.Dictionary(chunks)             => Some(ResponseValue.ObjectValue(chunks.flatMap { case (k, v) =>
          toResponseValue(v).map(toTyped[String](k).getOrElse(throw new Error("could not transform")) -> _)
        }.toList))
      case DynamicValue.Singleton(_)                   => None
      case DynamicValue.NoneValue                      => Some(Value.NullValue)
      case DynamicValue.DynamicAst(_)                  => None
      case DynamicValue.SetValue(_)                    => None
      case DynamicValue.Record(_, fields)              => Some(ResponseValue.ObjectValue(fields.flatMap { case (k, v) =>
          toResponseValue(v).map(k -> _)
        }.toList))
      case DynamicValue.Enumeration(_, _)              => None
      case DynamicValue.RightValue(_)                  => None
      case DynamicValue.SomeValue(input)               => toResponseValue(input)
      case DynamicValue.Tuple(_, _)                    => None
      case DynamicValue.LeftValue(_)                   => None
      case DynamicValue.Error(_)                       => None
    }
  }

  def toInputValue(input: DynamicValue): Option[InputValue] = {
    input match {
      case DynamicValue.Sequence(values) => Some(InputValue.ListValue(values.flatMap(toInputValue).toList))
      case DynamicValue.Primitive(value, standardType) => Some(toValue(value, standardType))
      case DynamicValue.Dictionary(chunks)             => Some(InputValue.ObjectValue(chunks.flatMap { case (k, v) =>
          toInputValue(v).map(toTyped[String](k).getOrElse(throw new Error("could not transform")) -> _)
        }.toMap))
      case DynamicValue.Singleton(_)                   => None
      case DynamicValue.NoneValue                      => Some(Value.NullValue)
      case DynamicValue.DynamicAst(_)                  => None
      case DynamicValue.SetValue(_)                    => None
      case DynamicValue.Record(_, b)                   => Some(InputValue.ObjectValue(b.flatMap { case (k, v) =>
          toInputValue(v).map(k -> _)
        }))
      case DynamicValue.Enumeration(_, _)              => None
      case DynamicValue.RightValue(_)                  => None
      case DynamicValue.SomeValue(input)               => toInputValue(input)
      case DynamicValue.Tuple(_, _)                    => None
      case DynamicValue.LeftValue(_)                   => None
      case DynamicValue.Error(_)                       => None
    }
  }

  def toTyped[A](d: DynamicValue)(implicit schema: Schema[A]): Option[A] = d.toTypedValueOption(schema)

  def getPath(d: DynamicValue, path: List[String]): Option[DynamicValue] =
    path match {
      case Nil          => Some(d)
      case head :: tail => d match {
          case DynamicValue.Record(_, b)  => b.get(head).flatMap(getPath(_, tail))
          case DynamicValue.SomeValue(a)  => getPath(a, path)
          case DynamicValue.Sequence(a)   => head.toIntOption.flatMap(a.lift).flatMap(getPath(_, tail))
          case DynamicValue.Dictionary(b) =>
            val stringTag = StandardType.StringType.asInstanceOf[StandardType[Any]]
            b.collect { case (DynamicValue.Primitive(`head`, `stringTag`), value) => value }.headOption
              .flatMap(getPath(_, tail))
          case _                          => None
        }
    }

  def fromResponseValue(input: ResponseValue): Option[DynamicValue] = {
    input match {
      case ResponseList(values)    => Some(DynamicValue(values.map(fromResponseValue(_))))
      case ResponseObject(fields)  => Some(DynamicValue(fields.toMap.map { case (k, v) => k -> fromResponseValue(v) }))
      case StringValue(value)      => Some(DynamicValue(value))
      case NullValue               => Some(DynamicValue.NoneValue)
      case BooleanValue(value)     => Some(DynamicValue(value))
      case BigDecimalNumber(value) => Some(DynamicValue(value))
      case DoubleNumber(value)     => Some(DynamicValue(value))
      case FloatNumber(value)      => Some(DynamicValue(value))
      case BigIntNumber(value)     => Some(DynamicValue(value))
      case IntNumber(value)        => Some(DynamicValue(value))
      case LongNumber(value)       => Some(DynamicValue(value))
      case EnumValue(_)            => None
      case StreamValue(_)          => None
    }
  }

  def fromInputValue(input: InputValue): Option[DynamicValue] = {
    input match {
      case InputList(values)       => Some(DynamicValue(values.map(fromInputValue(_))))
      case InputObject(fields)     => Some(DynamicValue(fields.map { case (k, v) => k -> fromInputValue(v) }))
      case StringValue(value)      => Some(DynamicValue(value))
      case NullValue               => Some(DynamicValue.NoneValue)
      case BooleanValue(value)     => Some(DynamicValue(value))
      case BigDecimalNumber(value) => Some(DynamicValue(value))
      case DoubleNumber(value)     => Some(DynamicValue(value))
      case FloatNumber(value)      => Some(DynamicValue(value))
      case BigIntNumber(value)     => Some(DynamicValue(value))
      case IntNumber(value)        => Some(DynamicValue(value))
      case LongNumber(value)       => Some(DynamicValue(value))
      case EnumValue(_)            => None
      case VariableValue(_)        => None
    }
  }

  def record(fields: (String, DynamicValue)*): DynamicValue =
    DynamicValue.Record(TypeId.Structural, ListMap.from(fields))

  def fromJson(json: Json): DynamicValue =
    json match {
      case Json.Obj(fields)   => DynamicValue
          .Record(TypeId.Structural, ListMap.from(fields.map { case (k, v) => k -> fromJson(v) }))
      case Json.Arr(elements) => DynamicValue(elements.map(fromJson))
      case Json.Bool(value)   => DynamicValue(value)
      case Json.Str(value)    => DynamicValue(value)
      case Json.Num(value)    => DynamicValue(value)
      case Json.Null          => DynamicValue.NoneValue
    }

  private def toJsonPrimitive[A](value: A, standardType: StandardType[A]): Json =
    standardType match {
      case StandardType.UnitType           => Json.Str(value.toString)
      case StandardType.StringType         => Json.Str(value.toString)
      case StandardType.BoolType           => Json.Bool(value.asInstanceOf[Boolean])
      case StandardType.ByteType           => Json.Str(value.toString)
      case StandardType.ShortType          => Json.Str(value.toString)
      case StandardType.IntType            => Json.Num(value.asInstanceOf[Int])
      case StandardType.LongType           => Json.Num(value.asInstanceOf[Long])
      case StandardType.FloatType          => Json.Num(value.asInstanceOf[Float])
      case StandardType.DoubleType         => Json.Num(value.asInstanceOf[Double])
      case StandardType.BinaryType         => Json
          .Str(java.util.Base64.getEncoder.encodeToString(value.asInstanceOf[Chunk[Byte]].toArray))
      case StandardType.CharType           => Json.Str(value.toString)
      case StandardType.UUIDType           => Json.Str(value.toString)
      case StandardType.BigDecimalType     => Json.Num(value.asInstanceOf[BigDecimalJava])
      case StandardType.BigIntegerType     => Json.Str(value.toString)
      case StandardType.DayOfWeekType      => Json.Str(value.toString)
      case StandardType.MonthType          => Json.Str(value.toString)
      case StandardType.MonthDayType       => Json.Str(value.toString)
      case StandardType.PeriodType         => Json.Str(value.toString)
      case StandardType.YearType           => Json.Str(value.toString)
      case StandardType.YearMonthType      => Json.Str(value.toString)
      case StandardType.ZoneIdType         => Json.Str(value.toString)
      case StandardType.ZoneOffsetType     => Json.Str(value.toString)
      case StandardType.DurationType       => Json.Str(value.toString)
      case StandardType.InstantType        => Json.Str(value.toString)
      case StandardType.LocalDateType      => Json.Str(value.toString)
      case StandardType.LocalTimeType      => Json.Str(value.toString)
      case StandardType.LocalDateTimeType  => Json.Str(value.toString)
      case StandardType.OffsetTimeType     => Json.Str(value.toString)
      case StandardType.OffsetDateTimeType => Json.Str(value.toString)
      case StandardType.ZonedDateTimeType  => Json.Str(value.toString)
    }

  def toJson(d: DynamicValue): Option[Json] =
    d match {
      case DynamicValue.Record(_, values)              => Some(Json.Obj(Chunk.from(values.flatMap { case (k, v) =>
          toJson(v).map(k -> _)
        })))
      case DynamicValue.Enumeration(_, (name, value))  => Some(Json.Obj(Chunk(toJson(value).map(name -> _)).flatten))
      case DynamicValue.Sequence(values)               => Some(Json.Arr(Chunk.from(values.flatMap(toJson))))
      case DynamicValue.Dictionary(_)                  => None
      case DynamicValue.SetValue(values)               => Some(Json.Arr(Chunk.from(values.flatMap(toJson))))
      case DynamicValue.Primitive(value, standardType) => Some(toJsonPrimitive(value, standardType))
      case DynamicValue.Singleton(_)                   => None
      case DynamicValue.SomeValue(value)               => toJson(value)
      case DynamicValue.NoneValue                      => Some(Json.Null)
      case DynamicValue.Tuple(left, right)             => Some(Json.Arr(Chunk(toJson(left), toJson(right)).flatten))
      case DynamicValue.LeftValue(value)               => toJson(value)
      case DynamicValue.RightValue(value)              => toJson(value)
      case DynamicValue.DynamicAst(_)                  => None
      case DynamicValue.Error(_)                       => None
    }
}
