import { Injectable } from '@angular/core';
import { TrainingRepository } from '../../data/repositories/training.repository';
import { UseCase } from '../../domain/interfaces/use-case.interface';

@Injectable({ providedIn: 'root' })
export class TriggerTrainingUseCase implements UseCase<void, void> {
  constructor(private trainingRepo: TrainingRepository) {}

  async execute(): Promise<void> {
    await this.trainingRepo.trigger();
  }
}
