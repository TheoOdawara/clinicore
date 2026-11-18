package fixture.core.model

import android.os.Parcelable

data class Patient(val name: String)

interface Shareable : Parcelable
