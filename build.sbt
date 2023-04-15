val calibanVersion = "2.0.2"
val zioVersion     = "2.0.12"
val zioJsonVersion = "0.4.2"
val rocksDB        = "0.4.2"

val zioSchema           = "dev.zio"               %% "zio-schema"            % "0.4.7"
val zioSchemaDerivation = "dev.zio"               %% "zio-schema-derivation" % "0.4.7"
val zioSchemaJson       = "dev.zio"               %% "zio-schema-json"       % "0.4.7"
val pprint              = "com.lihaoyi"           %% "pprint"                % "0.8.1"
val zio                 = "dev.zio"               %% "zio"                   % zioVersion
val caliban             = "com.github.ghostdogpr" %% "caliban"               % calibanVersion
val calibanTools        = "com.github.ghostdogpr" %% "caliban-tools"         % calibanVersion
val zioJson             = "dev.zio"               %% "zio-json"              % zioJsonVersion
val zioJsonYaml         = "dev.zio"               %% "zio-json-yaml"         % zioJsonVersion
val zioParser           = "dev.zio"               %% "zio-parser"            % "0.1.8"
val zioHttp             = "dev.zio"               %% "zio-http"              % "0.0.4"
val zioCLI              = "dev.zio"               %% "zio-cli"               % "0.4.0"
val fansi               = "com.lihaoyi"           %% "fansi"                 % "0.4.0"
val zioRedis            = "dev.zio"               %% "zio-redis"             % "0.2.0"
val zioTest             = "dev.zio"               %% "zio-test"              % zioVersion % Test
val zioTestSBT          = "dev.zio"               %% "zio-test-sbt"          % zioVersion % Test
///

lazy val root    = (project in file(".")).aggregate(runtime, server, cli, registry).settings(name := "tailcall")
lazy val runtime = (project in file("runtime")).settings(
  libraryDependencies ++= Seq(
    zioSchema,
    zioSchemaDerivation,
    zioSchemaJson,
    pprint,
    zio,
    caliban,
    calibanTools.exclude("com.softwaremill.sttp.client3", "async-http-client-backend-zio_2.13")
      .exclude("com.softwaremill.sttp.client3", "zio_2.13").exclude("com.github.ghostdogpr", "caliban-client_2.13")
      .exclude("dev.zio", "zio-config_2.13").exclude("dev.zio", "zio-config-magnolia_2.13")
      .exclude("org.slf4j", "slf4j-api"),
    zioJson,
    zioJsonYaml,
    zioParser,
    zioHttp,
  ),
  libraryDependencies ++= zioTestDependencies,
)

lazy val cli = (project in file("cli")).settings(libraryDependencies ++= zioTestDependencies ++ Seq(zio, zioCLI, fansi))
  .dependsOn(runtime, registry)

lazy val server = (project in file("server")).settings(libraryDependencies ++= zioTestDependencies ++ Seq(zio, zioHttp))
  .dependsOn(runtime, registry)

lazy val registry = (project in file("registry"))
  .settings(libraryDependencies ++= zioTestDependencies ++ Seq(zio, zioHttp, zioRedis)).dependsOn(runtime)

val scala2Version = "2.13.10"
val scala3Version = "3.2.2"

ThisBuild / scalaVersion                                   := scala2Version
ThisBuild / crossScalaVersions                             := Seq(scala2Version)
ThisBuild / coverageEnabled                                := true
ThisBuild / scalafixDependencies += "com.github.liancheng" %% "organize-imports" % "0.6.0"

ThisBuild / scalacOptions := Seq("-language:postfixOps", "-Ywarn-unused", "-Xfatal-warnings", "-deprecation")

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
addCommandAlias("tc", "cli/run")
enablePlugins(JavaAppPackaging)

ThisBuild / githubWorkflowBuild ++= Seq(
  WorkflowStep.Sbt(List("lintCheck"), name = Some("Lint"), cond = Some(s"matrix.scala == '${scala2Version}'"))
)

ThisBuild / githubWorkflowAddedJobs ++= Seq(WorkflowJob(
  "deploy",
  "Deploy",
  List(
    WorkflowStep.Checkout,
    WorkflowStep.Sbt(List("Docker/stage")),
    WorkflowStep.Run(commands = List("cp ./fly.toml target/docker/stage/")),
    WorkflowStep.Use(UseRef.Public("superfly", "flyctl-actions/setup-flyctl", "master")),
    WorkflowStep.Run(
      commands = List("flyctl deploy --remote-only ./target/docker/stage"),
      cond = Option("github.event_name == 'push' && github.ref == 'refs/heads/main'"),
      env = Map("FLY_API_TOKEN" -> "${{ secrets.FLY_API_TOKEN }}"),
    ),
  ),
  needs = List("build"),
  scalas = List(scala2Version),
))

ThisBuild / githubWorkflowPublishTargetBranches := Seq()

val zioTestDependencies = Seq(zioTest, zioTestSBT)

// The assembly merge settings
ThisBuild / assemblyMergeStrategy := { _ => MergeStrategy.first }

// Disable the main class discovery such that only the CLI is used as it's main class
// That way the executable script is only created for the CLI
Compile / discoveredMainClasses := (cli / Compile / mainClass).value.toSeq ++ (server / Compile / mainClass).value.toSeq

// The bash scripts classpath only needs the fat jar
// Script class path is used in stage command and not not docker stage
// So we add only the CLI application because only that's needed for the bash script
scriptClasspath := Seq((cli / assembly / assemblyJarName).value, (server / assembly / assemblyJarName).value)

// --- --- --- --- --- --- --- --- --- --- --- --- --- --- ---
// UNIVERSAL PACKAGE SETTINGS
// --- --- --- --- --- --- --- --- --- --- --- --- --- --- ---

// This is where we can add or remove files from the final package
Universal / mappings := {
  // The fat jar of the CLI
  val cliJar    = (cli / Compile / assembly).value
  val serverJar = (server / Compile / assembly).value

  // removing all the jars from the universal package
  val filtered = (Universal / mappings).value filter { case (file, name) => !name.endsWith(".jar") }

  // add only the cli fat jar
  filtered ++: Seq(cliJar -> ("lib/" + cliJar.getName), serverJar -> ("lib/" + serverJar.getName))
}

// --- --- --- --- --- --- --- --- --- --- --- --- --- --- ---
// DOCKER SETTINGS
// --- --- --- --- --- --- --- --- --- --- --- --- --- --- ---

// This is where we can add or remove files from the final package
Docker / mappings := {
  val serverJar = (server / Compile / assembly).value
  // removing means filtering
  val filtered  = (Docker / mappings).value.filter { case (file, name) => !name.endsWith(".jar") }

  // add the fat jar
  filtered ++: Seq(serverJar -> ("/opt/docker/lib/" + serverJar.getName))
}

maintainer         := "tushar@tailcall.in"
dockerCmd          := Seq("-Xmx200M", "-main", "tailcall.server.Main")
dockerBaseImage    := "eclipse-temurin:11"
dockerExposedPorts := Seq(8080)
