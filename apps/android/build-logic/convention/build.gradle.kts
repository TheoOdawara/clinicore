plugins { `kotlin-dsl` }

kotlin { jvmToolchain(21) }

dependencies {
    compileOnly(libs.android.gradle.plugin)
    compileOnly(libs.kotlin.gradle.plugin)
    compileOnly(libs.compose.compiler.gradle.plugin)
    implementation(libs.spotless.gradle.plugin)
    implementation(libs.detekt.gradle.plugin)
    implementation(libs.kover.gradle.plugin)

    testImplementation(libs.junit4)
    testImplementation(libs.truth)
}

gradlePlugin {
    plugins {
        register("androidApplication") {
            id = "clinicore.android.application"
            implementationClass = "AndroidApplicationConventionPlugin"
        }
        register("androidLibrary") {
            id = "clinicore.android.library"
            implementationClass = "AndroidLibraryConventionPlugin"
        }
        register("root") {
            id = "clinicore.root"
            implementationClass = "RootConventionPlugin"
        }
        register("androidCompose") {
            id = "clinicore.android.compose"
            implementationClass = "AndroidComposeConventionPlugin"
        }
    }
}
