package tailcall.runtime

import zio.Chunk
import zio.json.ast.Json
import zio.schema.{DynamicValue, StandardType}

/**
 * Can take in any input of type A and transform it into
 * another form of the same type. This way we can define
 * generic operations that can be applied over Json or
 * DynamicValue or any other type, all we need to provide is
 * an Accessor[A].
 */
sealed trait JsonTransformation[A] {
  self =>
  def transform(input: A)(implicit ev: JsonTransformation.Accessor[A]): A = JsonTransformation.transform(self, input)
  def apply(input: A)(implicit ev: JsonTransformation.Accessor[A]): A     = transform(input)
}

object JsonTransformation {
  final case class Identity[A]()                                          extends JsonTransformation[A]
  final case class Constant[A](value: A)                                  extends JsonTransformation[A]
  final case class ToPair[A]()                                            extends JsonTransformation[A]
  final case class Compose[A](list: List[JsonTransformation[A]])          extends JsonTransformation[A]
  final case class ApplySpec[A](spec: Map[String, JsonTransformation[A]]) extends JsonTransformation[A]
  final case class Path[A](list: List[String])                            extends JsonTransformation[A]

  trait Accessor[A] {
    def keys(a: A): Chunk[String]
    def values(a: A): Chunk[A] = keys(a).flatMap(key => get(a, key))
    def get(a: A, key: String): Option[A]
    def apply(a: Map[String, A]): A
    def apply(a: Chunk[A]): A
    def apply(a: String): A
    def apply(a: Int): A
    def apply(a: Long): A
    def apply(a: Boolean): A
    def toChunk(a: A): Option[Chunk[A]]
    def empty: A
  }

  object Accessor {
    def apply[A](implicit ev: Accessor[A]): Accessor[A] = ev
  }

  def transform[A](transformation: JsonTransformation[A], data: A)(implicit acc: Accessor[A]): A = {
    transformation match {
      case Identity() => data

      case Constant(value) => value

      case ToPair() => acc(data.keys.flatMap(key => data.get(key).map(value => acc(Chunk(acc(key), value)))))

      case Compose(list) => list match {
          case Nil          => data
          case head :: tail => Compose(tail).transform(head.transform(data))
        }

      case ApplySpec(spec) => data.toChunk match {
          case Some(list) => acc(list.map(transformation.transform(_)))
          case None       => acc {
              data.keys.foldLeft(Map.empty[String, A]) { case (obj, key) =>
                data.get(key) match {
                  case None        => obj
                  case Some(value) => spec.get(key) match {
                      case Some(transformation) => obj + (key -> transformation.transform(value))
                      case None                 => obj
                    }
                }
              }
            }
        }

      case Path(list) => list match {
          case Nil          => data
          case head :: tail => Path(tail).transform(data.get(head).getOrElse(acc.empty))
        }
    }
  }

  implicit final class AccessorSyntax[A](self: A) {
    def keys(implicit acc: Accessor[A]): Chunk[String]         = acc.keys(self)
    def values(implicit acc: Accessor[A]): Chunk[A]            = acc.values(self)
    def get(key: String)(implicit acc: Accessor[A]): Option[A] = acc.get(self, key)
    def toChunk(implicit acc: Accessor[A]): Option[Chunk[A]]   = acc.toChunk(self)
  }

  implicit val jsonAccessor: Accessor[Json] = new Accessor[Json] {
    override def keys(a: Json): Chunk[String] =
      a match {
        case Json.Obj(fields) => fields.map(_._1)
        case _                => Chunk.empty
      }

    override def get(a: Json, key: String): Option[Json] =
      a match {
        case Json.Obj(fields) => fields.collectFirst { case (`key`, value) => value }
        case _                => None
      }

    override def apply(a: Map[String, Json]): Json     = Json.Obj(Chunk.from(a))
    override def apply(a: Chunk[Json]): Json           = Json.Arr(a)
    override def toChunk(a: Json): Option[Chunk[Json]] =
      a match {
        case Json.Arr(elements) => Option(elements)
        case _                  => None
      }
    override def apply(a: String): Json                = Json.Str(a)
    override def apply(a: Int): Json                   = Json.Num(a)
    override def apply(a: Long): Json                  = Json.Num(a)
    override def apply(a: Boolean): Json               = Json.Bool(a)
    override def empty: Json                           = Json.Null
  }

  implicit val dynamicValueAccessor: Accessor[DynamicValue] = new Accessor[DynamicValue] {
    override def keys(a: DynamicValue): Chunk[String] =
      a match {
        case DynamicValue.Record(_, values)   => Chunk.fromIterable(values.keys)
        case DynamicValue.Dictionary(entries) => entries.collect {
            case (DynamicValue.Primitive(value, standardType: StandardType[_]), _)
                if standardType == StandardType.StringType => value.asInstanceOf[String]
          }
        case _                                => Chunk.empty
      }

    override def get(a: DynamicValue, key: String): Option[DynamicValue] =
      a match {
        case DynamicValue.Record(_, values)   => values.get(key)
        case DynamicValue.Dictionary(entries) => entries.find(_._1 == DynamicValue(key)).map(_._2)
        case _                                => None
      }

    override def apply(a: Map[String, DynamicValue]): DynamicValue = DynamicValue(a)

    override def apply(a: Chunk[DynamicValue]): DynamicValue = DynamicValue(a)

    override def apply(a: String): DynamicValue = DynamicValue(a)

    override def apply(a: Int): DynamicValue = DynamicValue(a)

    override def apply(a: Long): DynamicValue = DynamicValue(a)

    override def apply(a: Boolean): DynamicValue = DynamicValue(a)

    override def toChunk(a: DynamicValue): Option[Chunk[DynamicValue]] =
      a match {
        case DynamicValue.Sequence(values) => Option(values)
        case _                             => None
      }

    override def empty: DynamicValue = DynamicValue(Option.empty[Int])
  }
}
