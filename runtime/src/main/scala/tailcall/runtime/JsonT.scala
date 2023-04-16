package tailcall.runtime

import tailcall.runtime.internal.{DynamicValueUtil, JsonSchema}
import tailcall.runtime.transcoder.Transcoder
import zio.Chunk
import zio.json.ast.Json
import zio.json.{DeriveJsonCodec, JsonCodec, jsonHint}
import zio.schema.{DeriveSchema, DynamicValue, Schema, StandardType}

/**
 * Can take in any input of type A and transform it into
 * another form of the same type. This way we can define
 * generic operations that can be applied over Json or
 * DynamicValue or any other type, all we need to provide is
 * an Accessor[A].
 */

sealed trait JsonT {
  self =>
  def pipe(other: JsonT): JsonT                             = other compose self
  def compose(other: JsonT): JsonT                          = JsonT.Compose(List(self, other))
  def apply[A](input: A)(implicit ev: JsonT.Accessor[A]): A = run(input)
  def run[A](input: A)(implicit ev: JsonT.Accessor[A]): A   = JsonT.transform(self, input)
  def debug(prefix: String): JsonT                          = self >>> JsonT.debug(prefix)
  def >>>(other: JsonT): JsonT                              = other <<< self
  def <<<(other: JsonT): JsonT                              = self compose other
}

object JsonT {
  @jsonHint("identity")
  case object Identity extends JsonT

  @jsonHint("constant")
  final case class Constant(json: Json) extends JsonT
  object Constant {
    implicit val jsonCodec: JsonCodec[Constant] = JsonCodec(Json.encoder, Json.decoder).transform(Constant(_), _.json)
  }

  @jsonHint("toPair")
  case object ToPair extends JsonT

  @jsonHint("toKeyValue")
  case object ToKeyValue extends JsonT

  @jsonHint("compose")
  final case class Compose(list: List[JsonT]) extends JsonT
  object Compose {
    implicit val jsonCodec: JsonCodec[Compose] = JsonCodec[List[JsonT]].transform(Compose(_), _.list)
  }

  @jsonHint("applySpec")
  final case class ApplySpec(spec: Map[String, JsonT]) extends JsonT
  object ApplySpec {
    implicit val jsonCodec: JsonCodec[ApplySpec] = JsonCodec[Map[String, JsonT]].transform(ApplySpec(_), _.spec)
  }

  @jsonHint("objPath")
  final case class ObjectPath(spec: Map[String, List[String]]) extends JsonT
  object ObjectPath {
    implicit val jsonCodec: JsonCodec[ObjectPath] = JsonCodec[Map[String, List[String]]]
      .transform(ObjectPath(_), _.spec)
  }

  @jsonHint("omit")
  final case class Omit(keys: List[String]) extends JsonT
  object Omit {
    implicit val jsonCodec: JsonCodec[Omit] = JsonCodec[List[String]].transform(Omit(_), _.keys)
  }

  @jsonHint("path")
  final case class Path(list: List[String]) extends JsonT
  object Path {
    implicit val jsonCodec: JsonCodec[Path] = JsonCodec[List[String]].transform(Path(_), _.list)
  }

  @jsonHint("debug")
  final case class Debug(prefix: String) extends JsonT

  @jsonHint("map")
  final case class SeqMap(jsonT: JsonT) extends JsonT
  object SeqMap {
    implicit val jsonCodec: JsonCodec[SeqMap] = JsonCodec[JsonT].transform(SeqMap(_), _.jsonT)
  }

  @jsonHint("flatMap")
  final case class FlatMap(jsonT: JsonT) extends JsonT
  object FlatMap {
    implicit val jsonCodec: JsonCodec[FlatMap] = JsonCodec[JsonT].transform(FlatMap(_), _.jsonT)
  }

  def applySpec(spec: (String, JsonT)*): JsonT      = ApplySpec(spec.toMap)
  def const(json: Json): JsonT                      = Constant(json)
  def debug(prefix: String): JsonT                  = Debug(prefix)
  def identity: JsonT                               = Identity
  def map(jsonT: JsonT): JsonT                      = SeqMap(jsonT)
  def flatMap(jsonT: JsonT): JsonT                  = FlatMap(jsonT)
  def objPath(spec: (String, List[String])*): JsonT = ObjectPath(spec.toMap)
  def omit(keys: String*): JsonT                    = Omit(keys.toList)
  def path(list: String*): JsonT                    = Path(list.toList)
  def toKeyValue: JsonT                             = ToKeyValue
  def toPair: JsonT                                 = ToPair
  def compose(list: JsonT*): JsonT                  = Compose(list.toList)
  def pipe(list: JsonT*): JsonT                     = Compose(list.toList.reverse)

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
    def apply(a: Json): A
    def toChunk(a: A): Option[Chunk[A]]
    def empty: A
  }

  object Accessor {
    def apply[A](implicit ev: Accessor[A]): Accessor[A] = ev
  }

  def transform[A](transformation: JsonT, data: A)(implicit acc: Accessor[A]): A = {
    transformation match {
      case Identity => data

      case Constant(value) => acc(value)

      case ToPair => acc(data.keys.flatMap(key => data.get(key).map(value => acc(Chunk(acc(key), value)))))

      case Compose(seq) => seq.foldRight(data) { case (jsonT, data) => jsonT(data) }

      case ApplySpec(spec) => data.toChunk match {
          case Some(list) => acc(list.map(transformation.run(_)))
          case None       => acc {
              spec.keys.foldLeft(Map.empty[String, A]) { case (obj, key) =>
                spec.get(key) match {
                  case None        => obj
                  case Some(jsonT) => obj + (key -> jsonT(data))
                }
              }
            }
        }

      case ObjectPath(spec) => acc {
          spec.keys.foldLeft(Map.empty[String, A]) { case (obj, key) =>
            spec.get(key) match {
              case None       => obj
              case Some(list) => obj + (key -> Path(list).run(data))
            }
          }
        }

      case Path(list) => list match {
          case Nil          => data
          case head :: tail => Path(tail).run(data.get(head).getOrElse(acc.empty))
        }

      case ToKeyValue => acc(data.keys.flatMap { key =>
          data.get(key).map(value => acc(Map("key" -> acc(key), "value" -> value)))
        })

      case Debug(prefix) =>
        println(prefix + ": " + data)
        data

      case SeqMap(jsonT)  => data.toChunk match {
          case Some(list) => acc(list.map(jsonT(_)))
          case None       => acc(Chunk.empty)
        }
      case FlatMap(jsonT) => data.toChunk match {
          case Some(list) => acc(list.flatMap(jsonT(_).toChunk.getOrElse(Chunk.empty)))
          case None       => acc(Chunk.empty)
        }

      case Omit(keys) => acc(data.keyValue.foldLeft(Map.empty[String, A]) { case (map, (key, value)) =>
          if (keys.contains(key)) map else map + (key -> value)
        })
    }
  }

  implicit final class AccessorSyntax[A](self: A) {
    def keys(implicit acc: Accessor[A]): Chunk[String]          = acc.keys(self)
    def values(implicit acc: Accessor[A]): Chunk[A]             = acc.values(self)
    def keyValue(implicit acc: Accessor[A]): Chunk[(String, A)] =
      self.keys.flatMap(key => Chunk.fromIterable(self.get(key).map(key -> _)))
    def get(key: String)(implicit acc: Accessor[A]): Option[A]  = acc.get(self, key)
    def toChunk(implicit acc: Accessor[A]): Option[Chunk[A]]    = acc.toChunk(self)
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

    override def apply(a: Json): Json = a
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

    override def get(a: DynamicValue, key: String): Option[DynamicValue] = DynamicValueUtil.getPath(a, key :: Nil)

    override def apply(a: Map[String, DynamicValue]): DynamicValue = DynamicValueUtil.record(a.toSeq: _*)

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

    override def apply(a: Json): DynamicValue = Transcoder.toDynamicValue(a).get
  }

  implicit final private[JsonT] def jsonSchema: Schema[Json] = JsonSchema.schema
  implicit val jsonCodec: JsonCodec[JsonT]                   = DeriveJsonCodec.gen[JsonT]
  implicit def schema: Schema[JsonT]                         = DeriveSchema.gen[JsonT]
}
