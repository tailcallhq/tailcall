package tailcall.runtime.transcoder

import tailcall.runtime.transcoder.Transcoder.TExit
import zio.Chunk
import zio.json.ast.Json
import zio.schema.{DynamicValue, StandardType}

import java.math.{BigDecimal => BigDecimalJava}

object DynamicValue2JsonAST {
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

  def toJson(d: DynamicValue): TExit[String, Json] =
    d match {
      case DynamicValue.Record(_, values) => TExit.foreachChunk(Chunk.fromIterable(values)) { case (name, value) =>
          toJson(value).map(name -> _)
        }.map(list => Json.Obj(Chunk.from(list)))
      case DynamicValue.Enumeration(_, (name, value))  => toJson(value).map(value => Json.Obj(Chunk(name -> value)))
      case DynamicValue.Sequence(values)               => TExit.foreachChunk(values)(toJson(_))
          .map(values => Json.Arr(Chunk.from(values)))
      case DynamicValue.Dictionary(_)                  => TExit.fail("Can not transcoder Dictionary to a DynamicValue")
      case DynamicValue.SetValue(values)               => TExit.foreach(values.toList)(toJson(_))
          .map(values => Json.Arr(Chunk.from(values)))
      case DynamicValue.Primitive(value, standardType) => TExit.succeed(toJsonPrimitive(value, standardType))
      case DynamicValue.Singleton(_)                   => TExit.fail("Can not transcoder Singleton to a DynamicValue")
      case DynamicValue.SomeValue(value)               => toJson(value)
      case DynamicValue.NoneValue                      => TExit.succeed(Json.Null)
      case DynamicValue.Tuple(left, right)             => for {
          left  <- toJson(left)
          right <- toJson(right)
        } yield Json.Arr(Chunk(left, right))
      case DynamicValue.LeftValue(value)               => toJson(value)
      case DynamicValue.RightValue(value)              => toJson(value)
      case DynamicValue.DynamicAst(_)                  => TExit.fail("Can not transcoder DynamicAst to a DynamicValue")
      case DynamicValue.Error(_)                       => TExit.fail("Can not transcoder Error to a DynamicValue")
    }
}
