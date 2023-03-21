package tailcall.runtime

import tailcall.runtime.openApi.YamlParser
import tailcall.runtime.service.FileIO
import zio.ZIO
import zio.test.{ZIOSpecDefault, _}

import java.io.File

object OpenApiSpec extends ZIOSpecDefault {
  def spec =
    suite("OpenApiSpec")(test("parseyaml") {
      val program = for {
        fileIO  <- ZIO.service[FileIO]
        yaml    <- fileIO.read(new File(getClass.getResource("OpenAPI.yml").getPath))
        encoded <- ZIO.fromEither(YamlParser.parseFile(yaml))
        _ = pprint.pprintln(encoded)
      } yield ()
      assertCompletes
    })

}
