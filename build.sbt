val scala2Version = "2.13.10"
val scala3Version = "3.2.2"
val zioJson       = "0.4.2"
val zioSchema     = "0.4.7"
val caliban       = "2.0.2"
val zio           = "2.0.6"
val zioHttp       = "0.0.4"

ThisBuild / scalaVersion       := scala2Version
ThisBuild / crossScalaVersions := Seq(scala2Version, scala3Version)

ThisBuild / scalafixDependencies += "com.github.liancheng" %% "organize-imports" % "0.6.0"

ThisBuild / scalacOptions     := {
  Seq("-language:postfixOps") ++
    (CrossVersion.partialVersion(scalaVersion.value) match {
      case Some((2, _)) => Seq("-Ywarn-unused", "-Xfatal-warnings")
      case _            => Seq.empty
    })
}

ThisBuild / testFrameworks += new TestFramework("zio.test.sbt.ZTestFramework")
ThisBuild / Test / fork       := true
Global / semanticdbEnabled    := true
Global / onChangedBuildSource := ReloadOnSourceChanges

addCommandAlias("fmt", "scalafmt; Test / scalafmt; sFix;")
addCommandAlias("fmtCheck", "scalafmtCheck; Test / scalafmtCheck; sFixCheck")
addCommandAlias("sFix", "scalafixAll; Test / scalafixAll")
addCommandAlias("sFixCheck", "scalafixAll --check; Test / scalafixAll --check")
addCommandAlias("lint", "fmt; sFix")
addCommandAlias("lintCheck", "fmtCheck; sFixCheck")

ThisBuild / githubWorkflowBuild += WorkflowStep
  .Sbt(List("lintCheck"), name = Some("Lint"), cond = Some(s"matrix.scala == '${scala2Version}'"))
ThisBuild / githubWorkflowPublishTargetBranches := Seq()

lazy val root = (project in file(".")).aggregate(runtime, server)

lazy val runtime = (project in file("runtime")).settings(
  libraryDependencies := Seq(
    "dev.zio"               %% "zio-schema"            % zioSchema,
    "dev.zio"               %% "zio-schema-derivation" % zioSchema,
    "dev.zio"               %% "zio-schema-json"       % zioSchema,
    "com.lihaoyi"           %% "pprint"                % "0.8.1",
    "dev.zio"               %% "zio"                   % zio,
    "com.github.ghostdogpr" %% "caliban"               % caliban,
    "com.github.ghostdogpr" %% "caliban-tools"         % caliban,
    "dev.zio"               %% "zio-json"              % zioJson,
    "dev.zio"               %% "zio-json-yaml"         % zioJson,
    "dev.zio"               %% "zio-parser"            % "0.1.8",
    "io.netty"               % "netty-all"             % "4.1.68.Final",
    "dev.zio"               %% "zio-http"              % "0.0.4",

    // Testing
    "dev.zio" %% "zio-test"     % zio % Test,
    "dev.zio" %% "zio-test-sbt" % zio % Test
  )
)

lazy val server = (project in file("server")).settings(
  libraryDependencies := Seq(
    "dev.zio" %% "zio"         % zio,
    "dev.zio" %% "zio-http"    % zioHttp,
    "dev.zio" %% "zio-rocksdb" % "0.4.2"
  )
).dependsOn(runtime)
