import DesignSystem
import SwiftUI

struct RootView: View {
    var body: some View {
        NavigationStack {
            ScrollView {
                Text("Clinicore")
                    .font(ClinicoreFont.custom(size: 28, relativeTo: .largeTitle))
                    .fontWeight(.semibold)
                    .foregroundStyle(ClinicoreColor.primary)
                    .accessibilityAddTraits(.isHeader)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal)
            }
            .background(ClinicoreColor.background)
        }
    }
}

#Preview {
    RootView()
}
