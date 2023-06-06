val calibanVersion   = "2.2.1"
val zioVersion       = "2.0.15"
val zioJsonVersion   = "0.5.0"
val rocksDB          = "0.4.2"
val zioQuillVersion  = "4.6.1"
val zioSchemaVersion = "0.4.11"
val flywayVersion    = "9.19.1"

val caliban             = "com.github.ghostdogpr" %% "caliban"               % calibanVersion
val calibanTools        = "com.github.ghostdogpr" %% "caliban-tools"         % calibanVersion
val fansi               = "com.lihaoyi"           %% "fansi"                 % "0.4.0"
val flyway              = "org.flywaydb"           % "flyway-core"           % flywayVersion
val flywayMySQL         = "org.flywaydb"           % "flyway-mysql"          % flywayVersion
val mySQL               = "mysql"                  % "mysql-connector-java"  % "8.0.33"
val pprint              = "com.lihaoyi"           %% "pprint"                % "0.8.1"
val zio                 = "dev.zio"               %% "zio"                   % zioVersion
val zioCLI              = "dev.zio"               %% "zio-cli"               % "0.5.0"
val zioCache            = "dev.zio"               %% "zio-cache"             % "0.2.3"
val zioHttp             = "dev.zio"               %% "zio-http"              % "0.0.5"
val zioJson             = "dev.zio"               %% "zio-json"              % zioJsonVersion
val zioJsonYaml         = "dev.zio"               %% "zio-json-yaml"         % zioJsonVersion
val zioParser           = "dev.zio"               %% "zio-parser"            % "0.1.9"
val zioQuill            = "io.getquill"           %% "quill-zio"             % zioQuillVersion
val zioQuillJDBCZIO     = "io.getquill"           %% "quill-jdbc-zio"        % zioQuillVersion
val zioRedis            = "dev.zio"               %% "zio-redis"             % "0.2.0"
val zioSchema           = "dev.zio"               %% "zio-schema"            % zioSchemaVersion
val zioSchemaDerivation = "dev.zio"               %% "zio-schema-derivation" % zioSchemaVersion
val zioSchemaJson       = "dev.zio"               %% "zio-schema-json"       % zioSchemaVersion
val zioTest             = "dev.zio"               %% "zio-test"              % zioVersion
val zioTestSBT          = "dev.zio"               %% "zio-test-sbt"          % zioVersion

lazy val root = (project in file(".")).aggregate(runtime, server, cli, registry, testUtils).settings(name := "tailcall")

lazy val runtime = (project in file("runtime")).settings(
  resolvers +=
    "Sonatype OSS Snapshots" at "https://oss.sonatype.org/content/repositories/snapshots",
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
    zioCache,
  ),
  libraryDependencies ++= zioTestDependencies,
).dependsOn(testUtils % Test)

lazy val cli = (project in file("cli")).settings(libraryDependencies ++= zioTestDependencies ++ Seq(zio, zioCLI, fansi))
  .dependsOn(runtime, registry, testUtils % Test)

lazy val server = (project in file("server"))
  .settings(libraryDependencies ++= zioTestDependencies ++ Seq(zio, zioHttp, zioCLI))
  .dependsOn(runtime, registry, testUtils % Test)

lazy val registry = (project in file("registry")).settings(
  libraryDependencies ++= zioTestDependencies ++ Seq(
    zio,
    zioHttp,
    zioRedis,
    zioQuill,
    zioQuillJDBCZIO,
    mySQL,
    flyway,
    flywayMySQL,
    zioCLI,
  )
).dependsOn(runtime, testUtils % Test)

lazy val testUtils = (project in file("test-utils")).settings(libraryDependencies ++= Seq(zioTest, zioTestSBT))

val scala2Version = "2.13.11"
val scala3Version = "3.2.2"
val scalaVersions = List(scala2Version)
val javaVersions  = List(JavaSpec.temurin("20"))

ThisBuild / scalaVersion       := scala2Version
ThisBuild / crossScalaVersions := scalaVersions

ThisBuild / scalacOptions     := Seq("-language:postfixOps", "-Ywarn-unused", "-Xfatal-warnings", "-deprecation")
ThisBuild / testFrameworks += new TestFramework("zio.test.sbt.ZTestFramework")
ThisBuild / Test / fork       := true
Global / onChangedBuildSource := ReloadOnSourceChanges
Global / semanticdbEnabled    := true
Global / semanticdbVersion    := scalafixSemanticdb.revision
ThisBuild / githubWorkflowBuild ++= Seq(
  WorkflowStep.Sbt(List("lintCheck"), name = Some("Lint"), cond = Some(s"matrix.scala == '${scala2Version}'"))
)

ThisBuild / githubWorkflowJavaVersions          := javaVersions
ThisBuild / githubWorkflowBuild                 := {
  val mySQLWorkflowStep = WorkflowStep.Use(
    name = Option("Setup Mysql"),
    ref = UseRef.Public("mirromutth", "mysql-action", "v1.1"),
    params = Map(
      "mysql version"  -> "8.0",
      "mysql user"     -> "tailcall_main_user",
      "mysql database" -> "tailcall_main_db",
      "mysql password" -> "tailcall",
    ),
  )

  mySQLWorkflowStep +: (ThisBuild / githubWorkflowBuild).value
}

ThisBuild / githubWorkflowAddedJobs ++= {
  val githubWorkflowIsMain = Option("github.event_name == 'push' && github.ref == 'refs/heads/main'")
  val createReleaseId      = "create_release"
  val tagName              = "${{steps." + createReleaseId + ".outputs.tag_name}}"
  val fileName             = "tailcall-" + tagName + ".zip"
  Seq(
    // Deploy to fly.io
    WorkflowJob(
      "deploy",
      "Deploy",
      steps = List(
        WorkflowStep.Checkout,
        WorkflowStep.Sbt(List("Docker/stage")),
        WorkflowStep.Run(commands = List("cp ./fly.toml target/docker/stage/")),
        WorkflowStep.Use(UseRef.Public("superfly", "flyctl-actions/setup-flyctl", "master")),
        WorkflowStep.Run(
          commands = List("flyctl deploy --remote-only ./target/docker/stage"),
          cond = githubWorkflowIsMain,
          env = Map("FLY_API_TOKEN" -> "${{ secrets.FLY_API_TOKEN }}"),
        ),
      ),
      needs = List("build"),
      scalas = scalaVersions,
      javas = javaVersions,
    ),

    // Release to Github
    WorkflowJob(
      id = "release",
      name = "Release",
      cond = githubWorkflowIsMain,
      needs = List("build"),
      scalas = scalaVersions,
      javas = javaVersions,
      permissions = Option(sbtghactions.Permissions.Specify(Map(
        sbtghactions.PermissionScope.Contents     -> sbtghactions.PermissionValue.Write,
        sbtghactions.PermissionScope.PullRequests -> sbtghactions.PermissionValue.Write,
      ))),
      steps = List(
        WorkflowStep.Checkout,
        WorkflowStep.Sbt(commands = List("Universal/stage"), name = Option("Universal Stage")),
        WorkflowStep.Use(
          id = Option(createReleaseId),
          ref = UseRef.Public("release-drafter", "release-drafter", "v5"),
          params = Map("config-name" -> "release-drafter.yml"),
        ),
        WorkflowStep.Use(
          ref = UseRef.Public("TheDoctor0", "zip-release", "0.7.1"),
          params = Map(
            "type"       -> "zip",
            "filename"   -> fileName,
            "directory"  -> "target/universal/stage",
            "exclusions" -> "*.git*, .metals",
          ),
        ),
        WorkflowStep.Use(
          ref = UseRef.Public("softprops", "action-gh-release", "v1"),
          params = Map(
            "draft"       -> "true",
            "append_body" -> "true",
            "tag_name"    -> tagName,
            "files"       -> List("target/universal/stage/" + fileName).mkString("\n"),
          ),
        ),
      ),
    ),
  )
}

ThisBuild / githubWorkflowPublishTargetBranches := Seq()

addCommandAlias("fmt", "scalafmt; Test / scalafmt;")
addCommandAlias("fmtCheck", "scalafmtCheck; Test / scalafmtCheck; sFixCheck")
addCommandAlias("sFix", "scalafixAll; Test / scalafixAll")
addCommandAlias("sFixCheck", "scalafixAll --check; Test / scalafixAll --check")
addCommandAlias("lint", "fmt; sFix")
addCommandAlias("lintCheck", "fmtCheck; sFixCheck")
addCommandAlias("tc", "cli/run")
addCommandAlias("db", "registry/run")
enablePlugins(JavaAppPackaging)

val zioTestDependencies = Seq(zioTest % Test, zioTestSBT % Test)

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
