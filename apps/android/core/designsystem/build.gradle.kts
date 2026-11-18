plugins {
    alias(libs.plugins.clinicore.android.library)
    alias(libs.plugins.clinicore.android.compose)
}

android { namespace = "br.com.clinicore.core.designsystem" }

dependencies {
    api(libs.androidx.compose.material3)

    testImplementation(libs.junit4)
    testImplementation(libs.truth)
}
