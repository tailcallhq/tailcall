package tailcall.runtime.internal

import zio.schema.{DynamicValue, Schema, TypeId}

import scala.collection.immutable.ListMap

object DynamicValueUtil {
  def asString(dv: DynamicValue): Option[String] =
    dv match {
      case DynamicValue.Primitive(value, _) => Some(value.toString)
      case _                                => None
    }

  def toTyped[A](d: DynamicValue)(implicit schema: Schema[A]): Option[A] = d.toTypedValueOption(schema)

  def getPath(d: DynamicValue, paths: String*): Option[DynamicValue] = getPath(d, paths.toList)

  def getPath(d: DynamicValue, path: List[String], nestSeq: Boolean = false): Option[DynamicValue] =
    path match {
      case Nil          => Some(d)
      case head :: tail => d match {
          case DynamicValue.Record(_, b)  => b.get(head).flatMap(getPath(_, tail, nestSeq))
          case DynamicValue.SomeValue(a)  => getPath(a, path, nestSeq)
          case DynamicValue.Sequence(a)   =>
            if (nestSeq) Option(DynamicValue(a.map(getPath(_, path, nestSeq)).collect { case Some(a) => a }))
            else head.toIntOption.flatMap(a.lift).flatMap(getPath(_, tail, nestSeq))
          case DynamicValue.Dictionary(b) => b
              .collect { case (DynamicValue.Primitive(key, _), value) if key.toString == head => value }.headOption
              .flatMap(getPath(_, tail, nestSeq))
          case _                          => None
        }
    }

  def record(fields: (String, DynamicValue)*): DynamicValue =
    DynamicValue.Record(TypeId.Structural, ListMap.from(fields))
}
