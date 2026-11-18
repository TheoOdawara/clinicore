plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.kover)
    id("clinicore.root")
}

dependencies {
    kover(projects.app)
    kover(projects.core.designsystem)
}

kover {
    reports {
        filters { excludes { annotatedBy("androidx.compose.ui.tooling.preview.Preview") } }
    }
}
