package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{DynamicEval, Remote}

trait MapOps {
  implicit final class RemoteMapOps[A, B](val self: Remote[Map[A, B]]) {
    def get(key: Remote[A]): Remote[Option[B]] =
      Remote.unsafe.attempt(DynamicEval.mapGet(self.compile, key.compile))

    def getOrElse(key: Remote[A], default: Remote[B]): Remote[B] = get(key).getOrElse(default)

    def getOrDie(key: Remote[A]): Remote[B] = get(key).getOrDie

    def contains(key: Remote[A]): Remote[Boolean] = get(key).isSome

    def keys: Remote[Seq[A]] = ???

    def values: Remote[Seq[B]] = ???

    def mapValues[C](f: Remote[B] => Remote[C]): Remote[Map[A, C]] = ???

    def mapKeys[C](f: Remote[A] => Remote[C]): Remote[Map[C, B]] = ???

    def filterKeys(f: Remote[A] => Remote[Boolean]): Remote[Map[A, B]] = ???

    def filterValues(f: Remote[B] => Remote[Boolean]): Remote[Map[A, B]] = ???

    def filter(f: Remote[(A, B)] => Remote[Boolean]): Remote[Map[A, B]] = ???

    def map[C](f: Remote[(A, B)] => Remote[(A, C)]): Remote[Map[A, C]] = ???

    def flatMap[C](f: Remote[(A, B)] => Remote[Map[A, C]]): Remote[Map[A, C]] = ???
  }

}
