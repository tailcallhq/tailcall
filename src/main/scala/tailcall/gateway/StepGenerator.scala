package tailcall.gateway

import caliban.schema.Step
import caliban.{ResponseValue, Value}
import tailcall.gateway.StepGenerator.RemoteStep
import tailcall.gateway.ast.Orc.OExit
import tailcall.gateway.ast.{Context, Orc}
import tailcall.gateway.remote.{Remote, RemoteRuntime}
import zio.query.ZQuery
import zio.schema.{DynamicValue, StandardType}

final class StepGenerator(orc: Orc) {
  val nodeMap: Map[String, List[Orc.Field]] = orc
    .nodes
    .map(node => node.name -> node.fields)
    .toMap

  def gen(value: Any, standardType: StandardType[_]): Value =
    standardType match {
      case StandardType.StringType   => Value.StringValue(value.toString)
      case StandardType.IntType      => Value.IntValue(value.toString.toInt)
      case StandardType.MonthDayType => Value.StringValue(value.toString)
      case StandardType.LocalDateTimeType => Value.StringValue(value.toString)
      case StandardType.BoolType => Value.BooleanValue(value.toString.toBoolean)
      case StandardType.LocalTimeType      => Value.StringValue(value.toString)
      case StandardType.OffsetDateTimeType => Value.StringValue(value.toString)
      case StandardType.MonthType          => Value.StringValue(value.toString)
      case StandardType.ShortType      => Value.IntValue(value.toString.toShort)
      case StandardType.ZoneIdType     => Value.StringValue(value.toString)
      case StandardType.BigDecimalType => Value.StringValue(value.toString)
      case StandardType.YearType       => Value.IntValue(value.toString.toInt)
      case StandardType.ByteType       => Value.IntValue(value.toString.toByte)
      case StandardType.UUIDType       => Value.StringValue(value.toString)
      case StandardType.PeriodType     => Value.StringValue(value.toString)
      case StandardType.LongType       => Value.StringValue(value.toString)
      case StandardType.ZoneOffsetType => Value.StringValue(value.toString)
      case StandardType.BigIntegerType => Value.StringValue(value.toString)
      case StandardType.OffsetTimeType => Value.StringValue(value.toString)
      case StandardType.UnitType       => Value.NullValue
      case StandardType.DoubleType  => Value.FloatValue(value.toString.toDouble)
      case StandardType.InstantType => Value.StringValue(value.toString)
      case StandardType.FloatType   => Value.FloatValue(value.toString.toFloat)
      case StandardType.LocalDateType     => Value.StringValue(value.toString)
      case StandardType.ZonedDateTimeType => Value.StringValue(value.toString)
      case StandardType.YearMonthType     => Value.StringValue(value.toString)
      case StandardType.CharType          => Value.StringValue(value.toString)
      case StandardType.BinaryType        => Value.StringValue(
          java
            .util
            .Base64
            .getEncoder
            .encodeToString(value.asInstanceOf[Array[Byte]])
        )
      case StandardType.DurationType      => Value.StringValue(value.toString)
      case StandardType.DayOfWeekType     => Value.StringValue(value.toString)
    }

  def toValue(input: DynamicValue): ResponseValue = {
    input match {
      case DynamicValue.Sequence(values)               => ResponseValue
          .ListValue(values.map(toValue).toList)
      case DynamicValue.Primitive(value, standardType) =>
        gen(value, standardType)
      case DynamicValue.Dictionary(_)                  => ???
      case DynamicValue.Singleton(_)                   => ???
      case DynamicValue.NoneValue                      => ???
      case DynamicValue.DynamicAst(_)                  => ???
      case DynamicValue.SetValue(_)                    => ???
      case DynamicValue.Record(_, _)                   => ???
      case DynamicValue.Enumeration(_, _)              => ???
      case DynamicValue.RightValue(_)                  => ???
      case DynamicValue.SomeValue(_)                   => ???
      case DynamicValue.Tuple(_, _)                    => ???
      case DynamicValue.LeftValue(_)                   => ???
      case DynamicValue.Error(_)                       => ???
    }
  }

  def gen(
    name: String,
    fields: List[Orc.Field],
    context: Context
  ): RemoteStep = {

    Step.ObjectStep(
      name,
      fields.map(field => field.name -> gen(field.resolver, context)).toMap
    )
  }

  def gen(resolver: Orc.Resolver, context: Context): RemoteStep = {
    Step.QueryStep(ZQuery.fromZIO(
      resolver
        .remote(Remote(context))
        .evaluate
        .map {
          case OExit.Value(value) => Step.PureStep(toValue(value))
          case OExit.Ref(name)    => nodeMap(name) match {
              case fields => gen(name, fields, context)
              case Nil    => Step.NullStep
            }
        }
    ))
  }

  def gen: RemoteStep = {
    val context = Context(DynamicValue(()))

    orc.nodes.headOption match {
      case None        => Step.NullStep
      case Some(value) => gen(value.name, value.fields, context)
    }
  }
}

object StepGenerator {
  type RemoteStep = Step[RemoteRuntime]

  sealed trait GraphQLSchemaGeneratorError extends Throwable
  case object QueryNotFound                extends GraphQLSchemaGeneratorError
}
