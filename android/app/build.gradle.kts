plugins {
    id("com.android.application")
}

android {
    namespace = "local.aqelith.host"
    compileSdk = 35

    defaultConfig {
        applicationId = "local.aqelith.host"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
