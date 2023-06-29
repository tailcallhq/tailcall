package tailcall.runtime.internal

import better.files.File
import tailcall.runtime.internal.GraphQLTestSpec.GraphQLSpec
import zio.ZIO
import zio.test.Gen

trait GraphQLTestSpec {
  def graphQLSpecGen(dir: String): Gen[Any, GraphQLSpec] = Gen.fromZIO(load(dir).map(Gen.fromIterable(_))).flatten

  private def extractComponent(file: File, components: List[String], token: String): String = {
    val componentString = components.find(_.contains(token)).map(_.replace(token, "")).map(_.trim).getOrElse("").trim
    if (List("server-sdl", "client-sdl").contains(token) && componentString.isBlank())
      throw new Exception(s"${token} not found: ${file.path}")
    else componentString
  }

  private def load(dir: String): ZIO[Any, Nothing, List[GraphQLSpec]] =
    for {
      files       <- ZIO.succeedBlocking(File(getClass.getResource(dir)).glob("*.graphql").toList)
      contentList <- ZIO.foreach(files)(file => ZIO.succeedBlocking(file -> file.contentAsString))
    } yield contentList.map { case (file, content) =>
      val components = content.split("#>").map(_.trim)
      GraphQLSpec(
        file.name,
        extractComponent(file, components.toList, "server-sdl"),
        extractComponent(file, components.toList, "client-sdl"),
        extractComponent(file, components.toList, "client-error"),
      )
    }
}

object GraphQLTestSpec {
  final case class GraphQLSpec(name: String, serverSDL: String, clientSDL: String, clientError: String)
}
