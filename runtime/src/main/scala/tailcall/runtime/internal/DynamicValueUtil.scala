package tailcall.runtime.internal

import zio.schema.{DynamicValue, Schema, StandardType, TypeId}

import scala.collection.immutable.ListMap

object DynamicValueUtil {
  def asString(dv: DynamicValue): Option[String] =
    dv match {
      case DynamicValue.Primitive(value, _) => Some(value.toString)
      case _                                => None
    }

  def toTyped[A](d: DynamicValue)(implicit schema: Schema[A]): Option[A] = d.toTypedValueOption(schema)

  def getPath(d: DynamicValue, paths: String*): Option[DynamicValue] = getPath(d, paths.toList)

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

  def record(fields: (String, DynamicValue)*): DynamicValue =
    DynamicValue.Record(TypeId.Structural, ListMap.from(fields))
}
