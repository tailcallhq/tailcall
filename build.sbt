val calibanVersion   = "2.2.1"
val zioVersion       = "2.0.15"
val zioJsonVersion   = "0.6.0"
val rocksDB          = "0.4.2"
val zioQuillVersion  = "4.6.1"
val zioSchemaVersion = "0.4.13"
val flywayVersion    = "9.21.0"

val betterFiles         = "com.github.pathikrit"  %% "better-files"          % "3.9.2"
val caliban             = "com.github.ghostdogpr" %% "caliban"               % calibanVersion
val calibanTools        = "com.github.ghostdogpr" %% "caliban-tools"         % calibanVersion
val fansi               = "com.lihaoyi"           %% "fansi"                 % "0.4.0"
val flyway              = "org.flywaydb"           % "flyway-core"           % flywayVersion
val flywayMySQL         = "org.flywaydb"           % "flyway-mysql"          % flywayVersion
val mySQL               = "mysql"                  % "mysql-connector-java"  % "8.0.33"
val pprint              = "com.lihaoyi"           %% "pprint"                % "0.8.1"
val slf4j               = "org.slf4j"              % "slf4j-nop"             % "2.0.7"
val zio                 = "dev.zio"               %% "zio"                   % zioVersion
val zioCLI              = "dev.zio"               %% "zio-cli"               % "0.5.0"
val zioCache            = "dev.zio"               %% "zio-cache"             % "0.2.3"
val zioHttp             = "dev.zio"               %% "zio-http"              % "0.0.5"
val zioJson             = "dev.zio"               %% "zio-json"              % zioJsonVersion
val zioJsonYaml         = "dev.zio"               %% "zio-json-yaml"         % zioJsonVersion
val zioParser           = "dev.zio"               %% "zio-parser"            % "0.1.9"
val zioQuill            = "io.getquill"           %% "quill-zio"             % zioQuillVersion
val zioQuillJDBCZIO     = "io.getquill"           %% "quill-jdbc-zio"        % zioQuillVersion
val zioSchema           = "dev.zio"               %% "zio-schema"            % zioSchemaVersion
val zioSchemaDerivation = "dev.zio"               %% "zio-schema-derivation" % zioSchemaVersion
val zioSchemaJson       = "dev.zio"               %% "zio-schema-json"       % zioSchemaVersion
val zioTest             = "dev.zio"               %% "zio-test"              % zioVersion
val zioTestSBT          = "dev.zio"               %% "zio-test-sbt"          % zioVersion

lazy val root = (project in file(".")).aggregate(runtime, server, cli, registry, registryClient, testUtils)
  .settings(name := "tailcall")

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
    betterFiles,
  ),
  libraryDependencies ++= zioTestDependencies,
  buildInfoKeys    := Seq(name, version, scalaVersion, sbtVersion),
  buildInfoPackage := "tailcall",
  buildInfoOptions += BuildInfoOption.PackagePrivate,
  buildInfoOptions += BuildInfoOption.BuildTime,
  buildInfoOptions += BuildInfoOption.ToJson,
).dependsOn(testUtils % Test).enablePlugins(BuildInfoPlugin)

lazy val cli = (project in file("cli"))
  .settings(libraryDependencies ++= zioTestDependencies ++ Seq(zio, zioCLI, fansi, slf4j))
  .dependsOn(runtime, registryClient, testUtils % Test)

lazy val server = (project in file("server"))
  .settings(libraryDependencies ++= zioTestDependencies ++ Seq(zio, zioHttp, zioCLI, slf4j))
  .dependsOn(runtime, registry, testUtils % Test)

lazy val registry = (project in file("registry")).settings(
  libraryDependencies ++= zioTestDependencies ++ Seq(
    zio,
    zioHttp,
    zioQuill,
    zioQuillJDBCZIO,
    mySQL,
    flyway,
    flywayMySQL,
    zioCLI,
  )
).dependsOn(runtime, testUtils % Test)

lazy val registryClient = (project in file("registry-client"))
  .settings(libraryDependencies ++= zioTestDependencies ++ Seq(zio, zioHttp)).dependsOn(runtime, testUtils % Test)

lazy val testUtils = (project in file("test-utils")).settings(libraryDependencies ++= Seq(zioTest, zioTestSBT))

val scala2Version      = "2.13.11"
val scala3Version      = "3.2.2"
val scalaVersions      = List(scala2Version)
val defaultJavaVersion = JavaSpec.temurin("20")
val javaVersions       = List(defaultJavaVersion)
val appVersionEnv      = "APP_VERSION"

ThisBuild / scalaVersion       := scala2Version
ThisBuild / crossScalaVersions := scalaVersions
ThisBuild / version            := sys.env.getOrElse(appVersionEnv, "0.1.0-SNAPSHOT")
ThisBuild / scalacOptions      := Seq("-language:postfixOps", "-Ywarn-unused", "-Xfatal-warnings", "-deprecation")
ThisBuild / testFrameworks += new TestFramework("zio.test.sbt.ZTestFramework")
ThisBuild / Test / fork        := true
Global / onChangedBuildSource  := ReloadOnSourceChanges
Global / semanticdbEnabled     := true
Global / semanticdbVersion     := scalafixSemanticdb.revision
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

ThisBuild / githubWorkflowPermissions           := Option(
  sbtghactions.Permissions.Specify(Map(sbtghactions.PermissionScope.Contents -> sbtghactions.PermissionValue.Read))
)

ThisBuild / githubWorkflowAddedJobs ++= {
  val githubWorkflowIsMain    = Option("github.event_name == 'push' && github.ref == 'refs/heads/main'")
  val githubWorkflowIsRelease = Option(
    "github.event_name == 'release' && github.event.action == 'published' && startsWith(github.ref, 'refs/tags/v')"
  )
  val createReleaseId         = "create_release"
  val tagName                 = List("steps", createReleaseId, "outputs", "name").mkString("${{", ".", "}}")
  val releaseId               = List("steps", createReleaseId, "outputs", "id").mkString("${{", ".", "}}")
  val fileName                = "tailcall-" + tagName + ".zip"
  val jobPermissions          = sbtghactions.Permissions.Specify(Map(
    sbtghactions.PermissionScope.Contents     -> sbtghactions.PermissionValue.Write,
    sbtghactions.PermissionScope.PullRequests -> sbtghactions.PermissionValue.Write,
  ))
  val dockerContext           = "./target/docker/stage/"

  Seq(
    // Docker Stage
    WorkflowJob(
      "dockerStage",
      "Docker Stage",
      steps = List(WorkflowStep.Checkout, WorkflowStep.Sbt(List("Docker/stage"))),
      scalas = scalaVersions,
      javas = javaVersions,
      // FIXME: revert this change
      // cond = githubWorkflowIsMain,
      needs = List("build"),
    ),

    // Deploy to fly.io
    WorkflowJob(
      "deploy",
      "Deploy",
      steps = List(
        WorkflowStep.Checkout,
        WorkflowStep.Run(commands = List(s"cp ./fly.toml ${dockerContext}")),
        WorkflowStep.Use(UseRef.Public("superfly", "flyctl-actions/setup-flyctl", "master")),
        WorkflowStep.Run(
          commands = List(s"flyctl deploy --remote-only ${dockerContext} --wait-timeout 300"),
          env = Map("FLY_API_TOKEN" -> "${{ secrets.FLY_API_TOKEN }}"),
        ),
      ),
      needs = List("dockerStage"),
      scalas = scalaVersions,
      javas = javaVersions,
      cond = githubWorkflowIsMain,
    ),

    // Publish to ECR
    WorkflowJob(
      "dockerPublish",
      "Docker Publish",
      steps = List(
        WorkflowStep.Checkout,
        WorkflowStep.Use(
          id = Option("aws-ecr-action"),
          ref = UseRef.Public("kciter", "aws-ecr-action", "v4"),
          params = Map(
            "access_key_id"     -> "${{ secrets.AWS_ACCESS_KEY_ID }}",
            "secret_access_key" -> "${{ secrets.AWS_SECRET_ACCESS_KEY }}",
            "account_id"        -> "${{ secrets.AWS_ACCOUNT_ID }}",
            "repo"              -> "${{ vars.AWS_ECR_REPOSITORY }}",
            "region"            -> "${{ vars.AWS_REGION }}",
            "tags"              -> "${{ github.sha }}",
            "create_repo"       -> "true",
            "set_repo_policy"   -> "true",
            "path"              -> dockerContext,
          ),
        ),
        // FIXME: revert this change
        // needs = List("dockerStage"),
      ),
    ),

    // Release to Github
    WorkflowJob(
      id = "release",
      name = "Release",
      needs = List("build"),
      scalas = scalaVersions,
      javas = javaVersions,
      permissions = Option(jobPermissions),
      cond = githubWorkflowIsMain,
      steps = List(
        WorkflowStep.Checkout,
        WorkflowStep.Use(
          id = Option(createReleaseId),
          ref = UseRef.Public("release-drafter", "release-drafter", "v5"),
          params = Map("config-name" -> "release-drafter.yml"),
        ),
        WorkflowStep.Sbt(
          commands = List("Universal/stage"),
          name = Option("Universal Stage"),
          env = Map(appVersionEnv -> tagName),
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
          ref = UseRef.Public("xresloader", "upload-to-github-release", "v1"),
          params = Map(
            "release_id" -> releaseId,
            "file"       -> List("target/universal/stage/" + fileName).mkString(";"),
            "overwrite"  -> "true",
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
ThisBuild / assemblyMergeStrategy := {
  case PathList("META-INF", "services", _*) => MergeStrategy.concat
  case _                                    => MergeStrategy.first
}

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

maintainer         := "tushar@tailcall.run"
dockerCmd          := Seq(
  "-Xmx200M",
  "-main",
  "tailcall.server.Main",
  "--http-cache=1024",
  "--allowed-headers=cookie,authorization,apikey",
)
dockerBaseImage    := s"eclipse-temurin:${defaultJavaVersion.version}"
dockerExposedPorts := Seq(8080)
