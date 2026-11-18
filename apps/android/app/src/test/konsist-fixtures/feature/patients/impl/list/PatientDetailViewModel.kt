package fixture.feature.patients.impl.list

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import retrofit2.Retrofit

@HiltViewModel class PatientDetailViewModel : ViewModel()

data class PatientDetailUiState(val name: String, val visits: List<String>)

val client: Retrofit? = null
