lazy val root    = (project in file(".")).aggregate(runtime, server, cli, registry)
lazy val runtime = (project in file("runtime")).settings(
  libraryDependencies := Seq(
    "dev.zio"                %% "zio-schema"            % zioSchema,
    "dev.zio"                %% "zio-schema-derivation" % zioSchema,
    "dev.zio"                %% "zio-schema-json"       % zioSchema,
    "com.lihaoyi"            %% "pprint"                % "0.8.1",
    "dev.zio"                %% "zio"                   % zio,
    "com.github.ghostdogpr"  %% "caliban"               % caliban,
    ("com.github.ghostdogpr" %% "caliban-tools"         % caliban)
      .exclude("com.softwaremill.sttp.client3", "async-http-client-backend-zio_2.13")
      .exclude("com.softwaremill.sttp.client3", "zio_2.13").exclude("com.github.ghostdogpr", "caliban-client_2.13")
      .exclude("dev.zio", "zio-config_2.13").exclude("dev.zio", "zio-config-magnolia_2.13")
      .exclude("org.slf4j", "slf4j-api"),
    "dev.zio"                %% "zio-json"              % zioJson,
    "dev.zio"                %% "zio-json-yaml"         % zioJson,
    "dev.zio"                %% "zio-parser"            % "0.1.8",
    "dev.zio"                %% "zio-http"              % "0.0.4"
  ),
  libraryDependencies ++= zioTestDependencies
)

lazy val cli = (project in file("cli")).settings(
  libraryDependencies := zioTestDependencies ++ Seq(
    "dev.zio"     %% "zio"     % zio,
    "dev.zio"     %% "zio-cli" % "0.4.0",
    "com.lihaoyi" %% "fansi"   % "0.4.0"
  )
).dependsOn(runtime, registry)

lazy val server = (project in file("server")).settings(
  libraryDependencies := zioTestDependencies ++ Seq("dev.zio" %% "zio" % zio, "dev.zio" %% "zio-http" % zioHttp)
).dependsOn(runtime, registry)

lazy val registry = (project in file("registry")).settings(
  libraryDependencies := zioTestDependencies ++ Seq("dev.zio" %% "zio" % zio, "dev.zio" %% "zio-http" % zioHttp)
).dependsOn(runtime)

val scala2Version = "2.13.10"
val scala3Version = "3.2.2"
val zioJson       = "0.4.2"
val rocksDB       = "0.4.2"

ThisBuild / scalaVersion                                   := scala2Version
ThisBuild / crossScalaVersions                             := Seq(scala2Version)
ThisBuild / scalafixDependencies += "com.github.liancheng" %% "organize-imports" % "0.6.0"

ThisBuild / scalacOptions := Seq("-language:postfixOps", "-Ywarn-unused", "-Xfatal-warnings")

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

ThisBuild / githubWorkflowBuild ++= Seq(WorkflowStep.Sbt(
  List("lintCheck"),
  name = Some("Lint"),
  cond = Some(s"matrix.scala == '${scala2Version}'"),
  env = Map("SBT_NATIVE_CLIENT" -> "true")
))

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
      env = Map("FLY_API_TOKEN" -> "${{ secrets.FLY_API_TOKEN }}")
    )
  ),
  needs = List("build"),
  scalas = List(scala2Version)
))

ThisBuild / githubWorkflowPublishTargetBranches := Seq()
val zioSchema           = "0.4.7"
val caliban             = "2.0.2"
val zio                 = "2.0.10"
val zioHttp             = "0.0.4"
val zioTestDependencies = Seq("dev.zio" %% "zio-test" % zio % Test, "dev.zio" %% "zio-test-sbt" % zio % Test)

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
Docker / mappings   := {
  val serverJar = (server / Compile / assembly).value
  // removing means filtering
  val filtered  = (Docker / mappings).value.filter { case (file, name) => !name.endsWith(".jar") }

  // add the fat jar
  filtered ++: Seq(serverJar -> ("/opt/docker/lib/" + serverJar.getName))
}

Docker / maintainer := "tushar@tailcall.in"
dockerCmd           := Seq("-Xmx200M", "-main", "tailcall.server.Main")
dockerBaseImage     := "eclipse-temurin:11"
dockerExposedPorts  := Seq(8080)
