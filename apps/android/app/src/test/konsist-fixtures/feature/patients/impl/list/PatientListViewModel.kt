package fixture.feature.patients.impl.list

import androidx.lifecycle.ViewModel

class PatientListViewModel : ViewModel()

class PatientListPresenter : ViewModel()

data class PatientListUiState(var query: String, val names: MutableList<String>)

class PatientDao

class PatientResponse
